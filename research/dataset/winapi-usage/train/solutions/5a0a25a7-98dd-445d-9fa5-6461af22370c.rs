use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Networking::WebSocket::{WebSocketBeginServerHandshake, WEB_SOCKET_HANDLE};

fn call_web_socket_begin_server_handshake() -> WIN32_ERROR {
    let result = unsafe {
        WebSocketBeginServerHandshake(
            WEB_SOCKET_HANDLE(std::ptr::null_mut()),
            None,
            None,
            &[],
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };

    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or(WIN32_ERROR(0)),
    }
}
