use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Networking::WebSocket::{
    WebSocketCreateClientHandle, WEB_SOCKET_PROPERTY, WEB_SOCKET_PROPERTY_TYPE,
};

fn call_web_socket_create_client_handle() -> windows::Win32::Foundation::WIN32_ERROR {
    let properties = [WEB_SOCKET_PROPERTY {
        Type: WEB_SOCKET_PROPERTY_TYPE(1), // SECURE property type
        pvValue: std::ptr::null_mut(),
        ulValueSize: 0,
    }];

    unsafe {
        match WebSocketCreateClientHandle(&properties) {
            Ok(_) => WIN32_ERROR(0),
            Err(e) => {
                let code = e.code();
                WIN32_ERROR(code.0 as u32)
            }
        }
    }
}
