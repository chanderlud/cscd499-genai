use windows::core::Result;
use windows::Win32::Networking::WebSocket::{
    WebSocketCreateClientHandle, WEB_SOCKET_HANDLE, WEB_SOCKET_PROPERTY,
};

unsafe fn call_web_socket_create_client_handle() -> Result<WEB_SOCKET_HANDLE> {
    let properties: [WEB_SOCKET_PROPERTY; 0] = [];
    WebSocketCreateClientHandle(&properties)
}
