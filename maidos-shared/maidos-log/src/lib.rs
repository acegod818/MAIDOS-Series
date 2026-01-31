//! MAIDOS Tracing-based Logging System
//!
//! Provides structured logging with multiple output formats:
//! - Console (colored)
//! - File (JSON)
//! - Structured events with context
//!
//! # Example
//!
//! ```rust
//! use maidos_log::{info, error, span};
//! use tracing::Level;
//!
//! let id = "user-1";
//! let req_id = "req-1";
//! info!(user_id = %id, "User logged in");
//! let _span = span!(Level::INFO, "request", id = %req_id);
//! ```

use tracing::Level;
use tracing_subscriber::{
    fmt::{format::FmtSpan, time::ChronoLocal},
    filter::EnvFilter, prelude::*,
};

/// Result type for logging operations
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Initialize the logging system with default configuration
pub fn init() -> Result<()> {
    init_with_level(Level::INFO)
}

/// Initialize the logging system with a specific log level
pub fn init_with_level(level: Level) -> Result<()> {
    let filter = EnvFilter::from_default_env().add_directive(level.into());

    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_timer(ChronoLocal::rfc_3339())
                .with_span_events(FmtSpan::FULL)
                .with_filter(tracing_subscriber::filter::filter_fn(|meta| {
                    meta.target().starts_with("maidos") || meta.target().starts_with("maid_os")
                })),
        );

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

/// Log an INFO level message
#[macro_export]
macro_rules! info {
    (target: $target:expr, $($field:tt)*) => {
        tracing::info!(target: $target, $($field)*)
    };
    ($($arg:tt)*) => {
        tracing::info!($($arg)*)
    };
}

/// Log a WARN level message
#[macro_export]
macro_rules! warn {
    (target: $target:expr, $($field:tt)*) => {
        tracing::warn!(target: $target, $($field)*)
    };
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*)
    };
}

/// Log an ERROR level message
#[macro_export]
macro_rules! error {
    (target: $target:expr, $($field:tt)*) => {
        tracing::error!(target: $target, $($field)*)
    };
    ($($arg:tt)*) => {
        tracing::error!($($arg)*)
    };
}

/// Log a DEBUG level message
#[macro_export]
macro_rules! debug {
    (target: $target:expr, $($field:tt)*) => {
        tracing::debug!(target: $target, $($field)*)
    };
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*)
    };
}

/// Log a TRACE level message
#[macro_export]
macro_rules! trace {
    (target: $target:expr, $($field:tt)*) => {
        tracing::trace!(target: $target, $($field)*)
    };
    ($($arg:tt)*) => {
        tracing::trace!($($arg)*)
    };
}

/// Create a new span
#[macro_export]
macro_rules! span {
    ($lvl:expr, $name:expr) => {
        tracing::span!($lvl, $name)
    };
    ($lvl:expr, $name:expr, $($fields:tt)*) => {
        tracing::span!($lvl, $name, $($fields)*)
    };
}

/// Audit logging for security-sensitive operations
#[macro_export]
macro_rules! audit {
    ($op:expr, $user:expr, $details:expr) => {
        tracing::info!(
            target: "maidos.audit",
            operation = $op,
            user = $user,
            details = $details,
            "[AUDIT]"
        );
    };
}

/// Security event logging
#[macro_export]
macro_rules! security {
    ($event:expr, $severity:expr, $details:expr) => {
        tracing::warn!(
            target: "maidos.security",
            event = $event,
            severity = $severity,
            details = $details,
            "[SECURITY]"
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        // This test just ensures we can initialize without panicking
        let result = init_with_level(Level::ERROR);
        assert!(result.is_ok());
    }

    #[test]
    fn test_macros_compile() {
        // Test that macros compile without errors
        info!("Test info message");
        warn!("Test warn message");
        error!("Test error message");
        debug!("Test debug message");
        trace!("Test trace message");
        
        let _span = span!(Level::INFO, "test_span", test_field = "value");
        audit!("test_operation", "test_user", "test_details");
        security!("test_event", "high", "test_details");
    }
}
