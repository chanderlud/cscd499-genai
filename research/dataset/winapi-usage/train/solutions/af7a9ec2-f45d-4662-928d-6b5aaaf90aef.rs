use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Networking::WebSocket::{WebSocketAbortHandle, WEB_SOCKET_HANDLE};

fn call_web_socket_abort_handle() -> WIN32_ERROR {
    unsafe {
        WebSocketAbortHandle(WEB_SOCKET_HANDLE(std::ptr::null_mut()));
    }
    WIN32_ERROR(0)
}
