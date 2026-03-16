use std::ffi::OsStr;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use windows::core::{Result, Error, PCWSTR, HRESULT};
use windows::Win32::Networking::WinHttp::{
    WinHttpOpen, WinHttpConnect, WinHttpOpenRequest, WinHttpSendRequest,
    WinHttpReceiveResponse, WinHttpReadData, WinHttpCloseHandle,
    WINHTTP_ACCESS_TYPE_DEFAULT_PROXY, WINHTTP_OPEN_REQUEST_FLAGS,
};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

fn start_http_server(body: Vec<u8>) -> Result<(u16, thread::JoinHandle<()>)> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| Error::from_hresult(HRESULT::from_win32(e.raw_os_error().unwrap_or(0) as u32)))?;
    let port = listener.local_addr()
        .map_err(|e| Error::from_hresult(HRESULT::from_win32(e.raw_os_error().unwrap_or(0) as u32)))?
        .port();

    let handle = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            handle_connection(&mut stream, &body);
        }
    });

    Ok((port, handle))
}

fn handle_connection(stream: &mut TcpStream, body: &[u8]) {
    let mut buffer = [0; 1024];
    let _ = stream.read(&mut buffer);
    
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.write_all(body);
    let _ = stream.flush();
}

pub fn winhttp_get_from_local_server(body: &[u8]) -> Result<Vec<u8>> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
    }

    let (port, server_handle) = start_http_server(body.to_vec())?;
    
    let result = fetch_from_server(port);
    let _ = server_handle.join();
    result
}

fn fetch_from_server(port: u16) -> Result<Vec<u8>> {
    unsafe {
        let session = WinHttpOpen(
            PCWSTR(wide_null(OsStr::new("WinHTTP Client")).as_ptr()),
            WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
            PCWSTR::null(),
            PCWSTR::null(),
            0,
        );
        if session.is_null() {
            return Err(Error::from_thread());
        }

        let connect = WinHttpConnect(
            session,
            PCWSTR(wide_null(OsStr::new("127.0.0.1")).as_ptr()),
            port,
            0,
        );
        if connect.is_null() {
            WinHttpCloseHandle(session);
            return Err(Error::from_thread());
        }

        let request = WinHttpOpenRequest(
            connect,
            PCWSTR(wide_null(OsStr::new("GET")).as_ptr()),
            PCWSTR(wide_null(OsStr::new("/")).as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            std::ptr::null(),
            WINHTTP_OPEN_REQUEST_FLAGS(0),
        );
        if request.is_null() {
            WinHttpCloseHandle(connect);
            WinHttpCloseHandle(session);
            return Err(Error::from_thread());
        }

        let send_result = WinHttpSendRequest(
            request,
            None,
            None,
            0,
            0,
            0,
        );
        send_result?;

        let receive_result = WinHttpReceiveResponse(
            request,
            std::ptr::null_mut(),
        );
        receive_result?;

        let mut response_body = Vec::new();
        let mut buffer = [0u8; 1024];
        let mut bytes_read = 0u32;

        loop {
            let read_result = WinHttpReadData(
                request,
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u32,
                &mut bytes_read,
            );
            read_result?;

            if bytes_read == 0 {
                break;
            }

            response_body.extend_from_slice(&buffer[..bytes_read as usize]);
        }

        WinHttpCloseHandle(request);
        WinHttpCloseHandle(connect);
        WinHttpCloseHandle(session);

        Ok(response_body)
    }
}