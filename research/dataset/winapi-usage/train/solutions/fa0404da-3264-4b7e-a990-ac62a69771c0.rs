use windows::Win32::Networking::WebSocket::{WebSocketAbortHandle, WEB_SOCKET_HANDLE};

fn call_web_socket_abort_handle() -> windows::core::HRESULT {
    unsafe {
        WebSocketAbortHandle(WEB_SOCKET_HANDLE(std::ptr::null_mut()));
        windows::core::HRESULT(0)
    }
}
