//! C FFI exports for maidos-bus
//!
//! <impl>
//! WHAT: C-compatible FFI for P/Invoke from C#/.NET
//! WHY: Cross-language integration with MAIDOS applications
//! HOW: Blocking wrappers around async API, opaque pointers for handles
//! TEST: FFI handle creation, publish, receive
//! </impl>

use crate::event::Event;
use crate::publisher::{Publisher, PublisherConfig};
use crate::subscriber::{Subscriber, SubscriberConfig};
use std::ffi::{c_char, CStr, CString};
use std::ptr;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Opaque publisher handle
pub struct PublisherHandle {
    publisher: Publisher,
    runtime: Arc<Runtime>,
}

/// Opaque subscriber handle
pub struct SubscriberHandle {
    subscriber: Subscriber,
    runtime: Arc<Runtime>,
}

/// Event data for FFI
#[repr(C)]
pub struct FfiEvent {
    pub topic: *mut c_char,
    pub source: *mut c_char,
    pub payload: *mut u8,
    pub payload_len: usize,
    pub id: u64,
    pub timestamp: u64,
}

// ============================================================================
// Publisher FFI
// ============================================================================

/// Create a new publisher
///
/// # Safety
/// Returns null on failure
#[no_mangle]
pub unsafe extern "C" fn maidos_bus_publisher_create(bind_addr: *const c_char) -> *mut PublisherHandle {
    let addr = if bind_addr.is_null() {
        "127.0.0.1:0".to_string()
    } else {
        match CStr::from_ptr(bind_addr).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return ptr::null_mut(),
        }
    };

    let runtime = match Runtime::new() {
        Ok(rt) => Arc::new(rt),
        Err(_) => return ptr::null_mut(),
    };

    let config = PublisherConfig {
        bind_addr: addr,
        ..Default::default()
    };

    let mut publisher = Publisher::new(config);

    if runtime.block_on(publisher.start()).is_err() {
        return ptr::null_mut();
    }

    Box::into_raw(Box::new(PublisherHandle {
        publisher,
        runtime,
    }))
}

/// Get publisher bound port
///
/// # Safety
/// handle must be valid
#[no_mangle]
pub unsafe extern "C" fn maidos_bus_publisher_port(handle: *mut PublisherHandle) -> u16 {
    if handle.is_null() {
        return 0;
    }

    let h = &*handle;
    h.runtime
        .block_on(h.publisher.bound_addr())
        .map(|a| a.port())
        .unwrap_or(0)
}

/// Publish an event
///
/// # Safety
/// handle, topic, source must be valid pointers
#[no_mangle]
pub unsafe extern "C" fn maidos_bus_publish(
    handle: *mut PublisherHandle,
    topic: *const c_char,
    source: *const c_char,
    payload: *const u8,
    payload_len: usize,
) -> i32 {
    if handle.is_null() || topic.is_null() || source.is_null() {
        return -1;
    }

    let h = &mut *handle;

    let topic_str = match CStr::from_ptr(topic).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    let source_str = match CStr::from_ptr(source).to_str() {
        Ok(s) => s,
        Err(_) => return -3,
    };

    let payload_vec = if payload.is_null() || payload_len == 0 {
        vec![]
    } else {
        std::slice::from_raw_parts(payload, payload_len).to_vec()
    };

    let event = match Event::new(topic_str, source_str, payload_vec) {
        Ok(e) => e,
        Err(_) => return -4,
    };

    match h.runtime.block_on(h.publisher.publish(event)) {
        Ok(()) => 0,
        Err(_) => -5,
    }
}

/// Destroy publisher
///
/// # Safety
/// handle must be valid or null
#[no_mangle]
pub unsafe extern "C" fn maidos_bus_publisher_destroy(handle: *mut PublisherHandle) {
    if !handle.is_null() {
        let mut h = Box::from_raw(handle);
        let _ = h.runtime.block_on(h.publisher.stop());
    }
}

// ============================================================================
// Subscriber FFI
// ============================================================================

/// Create a new subscriber
///
/// # Safety
/// publisher_addr must be valid
#[no_mangle]
pub unsafe extern "C" fn maidos_bus_subscriber_create(
    publisher_addr: *const c_char,
) -> *mut SubscriberHandle {
    if publisher_addr.is_null() {
        return ptr::null_mut();
    }

    let addr = match CStr::from_ptr(publisher_addr).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return ptr::null_mut(),
    };

    let runtime = match Runtime::new() {
        Ok(rt) => Arc::new(rt),
        Err(_) => return ptr::null_mut(),
    };

    let config = SubscriberConfig {
        publisher_addr: addr,
        auto_reconnect: false,
        ..Default::default()
    };

    let mut subscriber = Subscriber::new(config);

    if runtime.block_on(subscriber.start()).is_err() {
        return ptr::null_mut();
    }

    Box::into_raw(Box::new(SubscriberHandle {
        subscriber,
        runtime,
    }))
}

/// Receive an event (blocking with timeout)
///
/// # Safety
/// handle must be valid, returns null if no event
#[no_mangle]
pub unsafe extern "C" fn maidos_bus_receive(
    handle: *mut SubscriberHandle,
    timeout_ms: u64,
) -> *mut FfiEvent {
    if handle.is_null() {
        return ptr::null_mut();
    }

    let h = &mut *handle;

    let result = h.runtime.block_on(async {
        tokio::time::timeout(
            tokio::time::Duration::from_millis(timeout_ms),
            h.subscriber.recv(),
        )
        .await
    });

    let event = match result {
        Ok(Some(e)) => e,
        _ => return ptr::null_mut(),
    };

    let topic = CString::new(event.topic.clone()).unwrap_or_default();
    let source = CString::new(event.source.clone()).unwrap_or_default();

    let payload_ptr = if event.payload.is_empty() {
        ptr::null_mut()
    } else {
        let mut payload = event.payload.clone();
        let ptr = payload.as_mut_ptr();
        std::mem::forget(payload);
        ptr
    };

    Box::into_raw(Box::new(FfiEvent {
        topic: topic.into_raw(),
        source: source.into_raw(),
        payload: payload_ptr,
        payload_len: event.payload.len(),
        id: event.id,
        timestamp: event.timestamp,
    }))
}

/// Free an event
///
/// # Safety
/// event must be valid or null
#[no_mangle]
pub unsafe extern "C" fn maidos_bus_event_free(event: *mut FfiEvent) {
    if !event.is_null() {
        let e = Box::from_raw(event);
        if !e.topic.is_null() {
            let _ = CString::from_raw(e.topic);
        }
        if !e.source.is_null() {
            let _ = CString::from_raw(e.source);
        }
        if !e.payload.is_null() && e.payload_len > 0 {
            let _ = Vec::from_raw_parts(e.payload, e.payload_len, e.payload_len);
        }
    }
}

/// Destroy subscriber
///
/// # Safety
/// handle must be valid or null
#[no_mangle]
pub unsafe extern "C" fn maidos_bus_subscriber_destroy(handle: *mut SubscriberHandle) {
    if !handle.is_null() {
        let mut h = Box::from_raw(handle);
        let _ = h.runtime.block_on(h.subscriber.stop());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_ffi_publisher_lifecycle() {
        unsafe {
            let addr = CString::new("127.0.0.1:0").unwrap();
            let handle = maidos_bus_publisher_create(addr.as_ptr());
            assert!(!handle.is_null());

            let port = maidos_bus_publisher_port(handle);
            assert!(port > 0);

            maidos_bus_publisher_destroy(handle);
        }
    }

    #[test]
    fn test_ffi_publish() {
        unsafe {
            let addr = CString::new("127.0.0.1:0").unwrap();
            let handle = maidos_bus_publisher_create(addr.as_ptr());
            assert!(!handle.is_null());

            let topic = CString::new("test.topic").unwrap();
            let source = CString::new("test-src").unwrap();
            let payload = [1u8, 2, 3];

            let result = maidos_bus_publish(
                handle,
                topic.as_ptr(),
                source.as_ptr(),
                payload.as_ptr(),
                payload.len(),
            );
            assert_eq!(result, 0);

            maidos_bus_publisher_destroy(handle);
        }
    }

    #[test]
    fn test_ffi_null_safety() {
        unsafe {
            // Null publisher
            assert_eq!(maidos_bus_publisher_port(ptr::null_mut()), 0);
            maidos_bus_publisher_destroy(ptr::null_mut()); // Should not crash

            // Null subscriber
            maidos_bus_subscriber_destroy(ptr::null_mut()); // Should not crash

            // Null event
            maidos_bus_event_free(ptr::null_mut()); // Should not crash
        }
    }

    #[test]
    fn test_ffi_subscriber_receive() {
        unsafe {
            let addr = CString::new("127.0.0.1:0").unwrap();
            let pub_handle = maidos_bus_publisher_create(addr.as_ptr());
            assert!(!pub_handle.is_null());

            let port = maidos_bus_publisher_port(pub_handle);
            assert!(port > 0);
            let sub_addr = CString::new(format!("127.0.0.1:{}", port)).unwrap();
            let sub_handle = maidos_bus_subscriber_create(sub_addr.as_ptr());
            assert!(!sub_handle.is_null());

            std::thread::sleep(std::time::Duration::from_millis(100));

            let topic = CString::new("test.topic").unwrap();
            let source = CString::new("test-src").unwrap();
            let payload = [9u8, 8, 7];
            let result = maidos_bus_publish(
                pub_handle,
                topic.as_ptr(),
                source.as_ptr(),
                payload.as_ptr(),
                payload.len(),
            );
            assert_eq!(result, 0);

            let event = maidos_bus_receive(sub_handle, 500);
            assert!(!event.is_null());
            maidos_bus_event_free(event);

            maidos_bus_subscriber_destroy(sub_handle);
            maidos_bus_publisher_destroy(pub_handle);
        }
    }

    #[test]
    fn test_ffi_publish_invalid_args() {
        unsafe {
            let result = maidos_bus_publish(ptr::null_mut(), ptr::null(), ptr::null(), ptr::null(), 0);
            assert_eq!(result, -1);
        }
    }

    #[test]
    fn test_ffi_subscriber_create_null_and_receive_null() {
        unsafe {
            let handle = maidos_bus_subscriber_create(ptr::null());
            assert!(handle.is_null());

            let event = maidos_bus_receive(ptr::null_mut(), 10);
            assert!(event.is_null());
        }
    }

    #[test]
    fn test_ffi_invalid_utf8_address() {
        unsafe {
            let invalid = [0xffu8, 0u8];
            let pub_handle = maidos_bus_publisher_create(invalid.as_ptr() as *const c_char);
            assert!(pub_handle.is_null());
        }
    }

    #[test]
    fn test_ffi_publisher_invalid_addr() {
        unsafe {
            let bad = CString::new("not-an-addr").unwrap();
            let pub_handle = maidos_bus_publisher_create(bad.as_ptr());
            assert!(pub_handle.is_null());
        }
    }

    #[test]
    fn test_ffi_receive_timeout_returns_null() {
        unsafe {
            let addr = CString::new("127.0.0.1:0").unwrap();
            let pub_handle = maidos_bus_publisher_create(addr.as_ptr());
            assert!(!pub_handle.is_null());

            let port = maidos_bus_publisher_port(pub_handle);
            let sub_addr = CString::new(format!("127.0.0.1:{}", port)).unwrap();
            let sub_handle = maidos_bus_subscriber_create(sub_addr.as_ptr());
            assert!(!sub_handle.is_null());

            let event = maidos_bus_receive(sub_handle, 10);
            assert!(event.is_null());

            maidos_bus_subscriber_destroy(sub_handle);
            maidos_bus_publisher_destroy(pub_handle);
        }
    }

    #[test]
    fn test_ffi_publish_invalid_topic_and_source() {
        unsafe {
            let addr = CString::new("127.0.0.1:0").unwrap();
            let handle = maidos_bus_publisher_create(addr.as_ptr());
            assert!(!handle.is_null());

            let invalid = [0xffu8, 0u8];
            let source = CString::new("src").unwrap();
            let result = maidos_bus_publish(
                handle,
                invalid.as_ptr() as *const c_char,
                source.as_ptr(),
                ptr::null(),
                0,
            );
            assert_eq!(result, -2);

            let topic = CString::new("topic").unwrap();
            let result = maidos_bus_publish(
                handle,
                topic.as_ptr(),
                invalid.as_ptr() as *const c_char,
                ptr::null(),
                0,
            );
            assert_eq!(result, -3);

            maidos_bus_publisher_destroy(handle);
        }
    }

    #[test]
    fn test_ffi_publish_invalid_topic_name() {
        unsafe {
            let addr = CString::new("127.0.0.1:0").unwrap();
            let handle = maidos_bus_publisher_create(addr.as_ptr());
            assert!(!handle.is_null());

            let topic = CString::new("").unwrap();
            let source = CString::new("src").unwrap();
            let result = maidos_bus_publish(
                handle,
                topic.as_ptr(),
                source.as_ptr(),
                ptr::null(),
                0,
            );
            assert_eq!(result, -4);

            maidos_bus_publisher_destroy(handle);
        }
    }

    #[test]
    fn test_ffi_receive_empty_payload() {
        unsafe {
            let addr = CString::new("127.0.0.1:0").unwrap();
            let pub_handle = maidos_bus_publisher_create(addr.as_ptr());
            assert!(!pub_handle.is_null());

            let port = maidos_bus_publisher_port(pub_handle);
            let sub_addr = CString::new(format!("127.0.0.1:{}", port)).unwrap();
            let sub_handle = maidos_bus_subscriber_create(sub_addr.as_ptr());
            assert!(!sub_handle.is_null());

            std::thread::sleep(std::time::Duration::from_millis(50));

            let topic = CString::new("test.empty").unwrap();
            let source = CString::new("test-src").unwrap();
            let result = maidos_bus_publish(
                pub_handle,
                topic.as_ptr(),
                source.as_ptr(),
                ptr::null(),
                0,
            );
            assert_eq!(result, 0);

            let event = maidos_bus_receive(sub_handle, 500);
            assert!(!event.is_null());
            let e = &*event;
            assert_eq!(e.payload_len, 0);
            assert!(e.payload.is_null());
            maidos_bus_event_free(event);

            maidos_bus_subscriber_destroy(sub_handle);
            maidos_bus_publisher_destroy(pub_handle);
        }
    }
}
