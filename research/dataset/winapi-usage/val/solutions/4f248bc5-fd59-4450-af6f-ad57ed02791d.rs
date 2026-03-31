use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Networking::WebSocket::{WebSocketBeginClientHandshake, WEB_SOCKET_HANDLE};

fn call_web_socket_begin_client_handshake() -> WIN32_ERROR {
    // SAFETY: We pass null handles and pointers as concrete values. The API will return an error,
    // which we safely convert to WIN32_ERROR. No memory is leaked or accessed unsafely.
    unsafe {
        match WebSocketBeginClientHandshake(
            WEB_SOCKET_HANDLE(std::ptr::null_mut()),
            None,
            None,
            None,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        ) {
            Ok(()) => WIN32_ERROR(0),
            Err(e) => WIN32_ERROR::from_error(&e).unwrap_or_default(),
        }
    }
}
