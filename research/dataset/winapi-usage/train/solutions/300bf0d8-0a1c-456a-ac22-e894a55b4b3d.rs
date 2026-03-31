use windows::core::{Error, Result};
use windows::Win32::Networking::WebSocket::{WebSocketAbortHandle, WEB_SOCKET_HANDLE};

fn call_web_socket_abort_handle() -> Result<()> {
    // SAFETY: Passing a null handle for this exercise. In production, a valid handle
    // obtained from WebSocketCreateClientHandle or WebSocketCreateServerHandle should be used.
    unsafe {
        WebSocketAbortHandle(WEB_SOCKET_HANDLE(std::ptr::null_mut()));
    }
    Ok(())
}
