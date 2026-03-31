use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Networking::WebSocket::{WebSocketCompleteAction, WEB_SOCKET_HANDLE};

fn call_web_socket_complete_action() -> WIN32_ERROR {
    unsafe {
        WebSocketCompleteAction(WEB_SOCKET_HANDLE(std::ptr::null_mut()), std::ptr::null(), 0);
    }
    WIN32_ERROR(0)
}
