#[allow(unused_imports)]
use windows::core::{Error, Result, HRESULT};
use windows::Win32::Networking::WebSocket::{WebSocketCompleteAction, WEB_SOCKET_HANDLE};

fn call_web_socket_complete_action() -> HRESULT {
    unsafe {
        WebSocketCompleteAction(WEB_SOCKET_HANDLE(std::ptr::null_mut()), std::ptr::null(), 0);
    }
    HRESULT(0)
}
