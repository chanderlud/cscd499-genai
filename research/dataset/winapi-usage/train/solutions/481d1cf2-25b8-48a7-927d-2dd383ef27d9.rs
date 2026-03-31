use windows::core::PCSTR;
use windows::core::{Error, Result};
use windows::Win32::Networking::WebSocket::{
    WebSocketBeginServerHandshake, WEB_SOCKET_HANDLE, WEB_SOCKET_HTTP_HEADER,
};

fn call_web_socket_begin_server_handshake() -> Result<()> {
    let hwebsocket = WEB_SOCKET_HANDLE(std::ptr::null_mut());
    let pszsubprotocolselected = PCSTR::null();
    let pszextensionselected: Option<&[PCSTR]> = None;
    let prequestheaders: &[WEB_SOCKET_HTTP_HEADER] = &[];
    let mut presponseheaders: *mut WEB_SOCKET_HTTP_HEADER = std::ptr::null_mut();
    let mut pulresponseheadercount: u32 = 0;

    // SAFETY: We pass correctly typed pointers and slices to the Win32 API.
    // The call may fail at runtime due to the dummy handle, which is propagated via `?`.
    unsafe {
        WebSocketBeginServerHandshake(
            hwebsocket,
            pszsubprotocolselected,
            pszextensionselected,
            prequestheaders,
            &mut presponseheaders,
            &mut pulresponseheadercount,
        )?;
    }
    Ok(())
}
