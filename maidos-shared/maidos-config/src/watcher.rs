//! File watcher for hot-reload support
//!
//! <impl>
//! WHAT: Watch config file for changes, trigger reload callback
//! WHY: Runtime config updates without restart
//! HOW: notify crate for cross-platform file watching, debounce rapid changes
//! TEST: File modification triggers callback, debouncing works
//! </impl>

use crate::error::{ConfigError, Result};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Handle to stop watching
pub struct WatchHandle {
    stop_tx: Sender<()>,
    thread_handle: Option<JoinHandle<()>>,
}

impl WatchHandle {
    /// Stop watching the file
    pub fn stop(mut self) {
        let _ = self.stop_tx.send(());
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for WatchHandle {
    fn drop(&mut self) {
        let _ = self.stop_tx.send(());
    }
}

/// Configuration for the file watcher
pub struct WatchConfig {
    /// Debounce duration - ignore rapid successive changes
    pub debounce_ms: u64,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self { debounce_ms: 100 }
    }
}

/// Watch a configuration file for changes
///
/// # Arguments
/// * `path` - Path to the config file to watch
/// * `callback` - Function called when file changes (receives the path)
/// * `watch_config` - Optional watcher configuration
///
/// # Returns
/// * `Ok(WatchHandle)` - Handle to stop watching
/// * `Err(ConfigError)` - If watcher setup fails
pub fn watch_file<F>(path: &Path, callback: F, watch_config: Option<WatchConfig>) -> Result<WatchHandle>
where
    F: Fn(&Path) + Send + 'static,
{
    let path = path.to_path_buf();
    let config = watch_config.unwrap_or_default();
    let debounce_duration = Duration::from_millis(config.debounce_ms);

    // Verify file exists
    if !path.exists() {
        return Err(ConfigError::FileNotFound(path));
    }

    // Get parent directory to watch (some systems don't notify on file changes directly)
    let watch_path = path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let (stop_tx, stop_rx) = channel::<()>();
    let (event_tx, event_rx) = channel::<notify::Result<Event>>();

    // Create watcher
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let _ = event_tx.send(res);
        },
        Config::default(),
    )
    .map_err(|e| ConfigError::WatcherError(e.to_string()))?;

    // Start watching
    watcher
        .watch(&watch_path, RecursiveMode::NonRecursive)
        .map_err(|e| ConfigError::WatcherError(e.to_string()))?;

    info!("Started watching config file: {}", path.display());

    // Spawn watcher thread
    let thread_handle = thread::spawn(move || {
        let _watcher = watcher; // Keep watcher alive
        let mut last_event: Option<Instant> = None;

        loop {
            // Check for stop signal (non-blocking)
            if stop_rx.try_recv().is_ok() {
                debug!("Watcher stop signal received");
                break;
            }

            // Check for file events (with timeout)
            match event_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(Ok(event)) => {
                    // Check if this event is for our file
                    let is_our_file = event.paths.iter().any(|p| {
                        p.file_name() == path.file_name()
                    });

                    if is_our_file && matches!(event.kind, notify::EventKind::Modify(_) | notify::EventKind::Create(_)) {
                        // Debounce: only trigger if enough time has passed
                        let now = Instant::now();
                        let should_trigger = match last_event {
                            Some(last) => now.duration_since(last) >= debounce_duration,
                            None => true,
                        };

                        if should_trigger {
                            last_event = Some(now);
                            info!("Config file changed: {}", path.display());
                            callback(&path);
                        } else {
                            debug!("Debouncing config change event");
                        }
                    }
                }
                Ok(Err(e)) => {
                    warn!("File watch error: {}", e);
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Normal timeout, continue loop
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    debug!("Event channel disconnected");
                    break;
                }
            }
        }

        info!("Config watcher stopped");
    });

    Ok(WatchHandle {
        stop_tx,
        thread_handle: Some(thread_handle),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tempfile::TempDir;

    #[test]
    fn test_watch_nonexistent_file() {
        let result = watch_file(Path::new("/nonexistent/file.toml"), |_| {}, None);
        assert!(matches!(result, Err(ConfigError::FileNotFound(_))));
    }

    #[test]
    fn test_watch_and_modify() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.toml");

        // Create initial file
        fs::write(&config_path, "[maidos]\nversion = \"1.0\"").unwrap();

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let handle = watch_file(
            &config_path,
            move |_| {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            },
            Some(WatchConfig { debounce_ms: 50 }),
        )
        .unwrap();

        // Give watcher time to start
        thread::sleep(Duration::from_millis(100));

        // Modify file
        fs::write(&config_path, "[maidos]\nversion = \"2.0\"").unwrap();

        // Wait for callback
        thread::sleep(Duration::from_millis(200));

        // Stop watcher
        handle.stop();

        // Verify callback was called at least once
        assert!(counter.load(Ordering::SeqCst) >= 1);
    }
}
