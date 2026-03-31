use windows::core::{Error, Result};
use windows::Win32::Networking::WebSocket::{WebSocketBeginClientHandshake, WEB_SOCKET_HANDLE};

fn call_web_socket_begin_client_handshake() -> Result<Result<()>> {
    let hwebsocket = WEB_SOCKET_HANDLE(std::ptr::null_mut());
    Ok(unsafe {
        WebSocketBeginClientHandshake(
            hwebsocket,
            None,
            None,
            None,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    })
}
