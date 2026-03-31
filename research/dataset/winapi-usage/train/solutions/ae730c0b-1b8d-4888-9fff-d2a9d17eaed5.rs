use windows::core::{Error, Result, HRESULT};
use windows::Win32::Networking::WebSocket::{
    WebSocketBeginServerHandshake, WEB_SOCKET_HANDLE, WEB_SOCKET_HTTP_HEADER,
};

fn call_web_socket_begin_server_handshake() -> HRESULT {
    unsafe {
        WebSocketBeginServerHandshake(
            WEB_SOCKET_HANDLE(std::ptr::null_mut()),
            windows::core::PCSTR::null(),
            None,
            &[] as &[WEB_SOCKET_HTTP_HEADER],
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
        .map(|_| HRESULT::default())
        .unwrap_or_else(|e| e.code())
    }
}
