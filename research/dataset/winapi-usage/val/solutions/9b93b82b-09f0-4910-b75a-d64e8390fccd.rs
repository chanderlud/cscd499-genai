use windows::core::{Error, Result, HRESULT};
use windows::Win32::Networking::WebSocket::{
    WebSocketBeginClientHandshake, WEB_SOCKET_HANDLE, WEB_SOCKET_HTTP_HEADER,
};

fn call_web_socket_begin_client_handshake() -> HRESULT {
    let mut additional_headers: *mut WEB_SOCKET_HTTP_HEADER = std::ptr::null_mut();
    let mut additional_header_count: u32 = 0;

    let result: Result<()> = unsafe {
        WebSocketBeginClientHandshake(
            WEB_SOCKET_HANDLE(std::ptr::null_mut()),
            None,
            None,
            None,
            &mut additional_headers,
            &mut additional_header_count,
        )
    };

    match result {
        Ok(()) => HRESULT::default(),
        Err(e) => e.code(),
    }
}
