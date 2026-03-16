use std::mem::{self, size_of, zeroed};
use std::ptr::{null, null_mut};

use windows::Win32::Foundation::{
    CloseHandle, ERROR_IO_PENDING, GetLastError, HANDLE, INVALID_HANDLE_VALUE, WAIT_TIMEOUT,
};
use windows::Win32::Networking::WinSock::*;
use windows::Win32::System::IO::{CreateIoCompletionPort, GetQueuedCompletionStatus, OVERLAPPED};
use windows::core::{BOOL, Error, GUID, HRESULT, PSTR, Result};

fn win32_error(code: u32) -> Error {
    Error::from_hresult(HRESULT::from_win32(code))
}

fn wsa_error(code: i32) -> Error {
    Error::from_hresult(HRESULT::from_win32(code as u32))
}

fn last_win32_error() -> Error {
    // SAFETY: thread-local getter
    win32_error(unsafe { GetLastError().0 })
}

fn last_wsa_error() -> Error {
    // SAFETY: thread-local getter for Winsock APIs
    wsa_error(unsafe { WSAGetLastError().0 })
}

fn check_wsa_zero(rc: i32) -> Result<()> {
    if rc == SOCKET_ERROR {
        Err(last_wsa_error())
    } else {
        Ok(())
    }
}

fn check_wsa_overlapped_i32(rc: i32) -> Result<()> {
    if rc == 0 {
        Ok(())
    } else {
        // SAFETY: valid immediately after failed Winsock call
        let err = unsafe { WSAGetLastError() };
        if err.0 == ERROR_IO_PENDING.0 as i32 {
            Ok(())
        } else {
            Err(wsa_error(err.0))
        }
    }
}

fn check_wsa_overlapped_bool(ok: BOOL) -> Result<()> {
    if ok.as_bool() {
        Ok(())
    } else {
        // SAFETY: valid immediately after failed Winsock call
        let err = unsafe { WSAGetLastError() };
        if err.0 == ERROR_IO_PENDING.0 as i32 {
            Ok(())
        } else {
            Err(wsa_error(err.0))
        }
    }
}

fn new_tcp_socket() -> Result<SOCKET> {
    // SAFETY: valid parameters for overlapped TCP socket
    unsafe {
        WSASocketW(
            AF_INET.0.into(),
            SOCK_STREAM.0,
            IPPROTO_TCP.0,
            None,
            0,
            WSA_FLAG_OVERLAPPED,
        )
    }
}

fn sockaddr_loopback(port: u16) -> SOCKADDR_IN {
    SOCKADDR_IN {
        sin_family: AF_INET,
        sin_port: port.to_be(),
        sin_addr: IN_ADDR {
            S_un: IN_ADDR_0 {
                S_addr: u32::from_ne_bytes([127, 0, 0, 1]),
            },
        },
        sin_zero: [0; 8],
    }
}

fn socket_opt_bytes<T>(value: &T) -> &[u8] {
    // SAFETY: plain byte view over a POD value for setsockopt
    unsafe { std::slice::from_raw_parts((value as *const T).cast::<u8>(), size_of::<T>()) }
}

struct WinsockGuard;
impl WinsockGuard {
    fn new() -> Result<Self> {
        let mut data = WSADATA::default();
        // SAFETY: valid parameters
        let rc = unsafe { WSAStartup(0x202, &mut data) };
        if rc != 0 {
            return Err(wsa_error(rc));
        }
        Ok(Self)
    }
}
impl Drop for WinsockGuard {
    fn drop(&mut self) {
        // SAFETY: paired with successful WSAStartup
        unsafe { WSACleanup() };
    }
}

struct SocketGuard(SOCKET);
impl Drop for SocketGuard {
    fn drop(&mut self) {
        if self.0 != INVALID_SOCKET {
            // SAFETY: valid socket or INVALID_SOCKET checked above
            unsafe {
                let _ = closesocket(self.0);
            }
        }
    }
}

struct IocpGuard(HANDLE);
impl Drop for IocpGuard {
    fn drop(&mut self) {
        // SAFETY: handle came from CreateIoCompletionPort on success
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

fn associate_socket_with_iocp(iocp: HANDLE, socket: SOCKET, key: usize) -> Result<()> {
    // SAFETY: sockets can be associated with an IOCP by casting SOCKET to HANDLE
    unsafe {
        CreateIoCompletionPort(HANDLE(socket.0 as *mut _), Some(iocp), key, 0)?;
    }
    Ok(())
}

type AcceptExFn = unsafe extern "system" fn(
    SOCKET,
    SOCKET,
    *mut std::ffi::c_void,
    u32,
    u32,
    u32,
    *mut u32,
    *mut OVERLAPPED,
) -> BOOL;

type ConnectExFn = unsafe extern "system" fn(
    SOCKET,
    *const SOCKADDR,
    i32,
    *const std::ffi::c_void,
    u32,
    *mut u32,
    *mut OVERLAPPED,
) -> BOOL;

fn get_extension_function<T>(socket: SOCKET, guid: GUID) -> Result<T> {
    let mut func_ptr: *mut std::ffi::c_void = null_mut();
    let mut bytes_returned = 0u32;

    // SAFETY: valid request buffer and output buffer
    let rc = unsafe {
        WSAIoctl(
            socket,
            SIO_GET_EXTENSION_FUNCTION_POINTER,
            Some(&guid as *const _ as *const std::ffi::c_void),
            size_of::<GUID>() as u32,
            Some(&mut func_ptr as *mut _ as *mut std::ffi::c_void),
            size_of::<T>() as u32,
            &mut bytes_returned,
            None,
            None,
        )
    };

    if rc == SOCKET_ERROR {
        return Err(last_wsa_error());
    }

    // SAFETY: WSAIoctl returned success, so the function pointer is valid for this extension
    Ok(unsafe { mem::transmute_copy(&func_ptr) })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum Operation {
    Accept,
    Connect,
    ClientSend,
    ClientRecv,
    ServerRecv,
    ServerSend,
}

#[repr(C)]
struct OverlappedContext {
    overlapped: OVERLAPPED,
    op: Operation,
    buffer: Vec<u8>,
    wsabuf: WSABUF,
}

impl OverlappedContext {
    fn new(op: Operation, buffer_size: usize) -> Self {
        let mut buffer = vec![0u8; buffer_size];
        let ptr = if buffer_size == 0 {
            null_mut()
        } else {
            buffer.as_mut_ptr()
        };

        Self {
            overlapped: unsafe { zeroed() },
            op,
            buffer,
            wsabuf: WSABUF {
                len: buffer_size as u32,
                buf: PSTR(ptr),
            },
        }
    }
}

pub fn iocp_tcp_echo(payload: &[u8]) -> Result<Vec<u8>> {
    if payload.is_empty() {
        return Ok(Vec::new());
    }

    const IOCP_TIMEOUT_MS: u32 = 5_000;
    const ACCEPT_ADDR_SPACE: usize = (size_of::<SOCKADDR_IN>() + 16) * 2;

    let _winsock = WinsockGuard::new()?;

    let listener = SocketGuard(new_tcp_socket()?);
    let accepted = SocketGuard(new_tcp_socket()?);
    let client = SocketGuard(new_tcp_socket()?);

    let listen_addr = sockaddr_loopback(0);

    // Bind listener to loopback on an ephemeral port.
    check_wsa_zero(unsafe {
        bind(
            listener.0,
            &listen_addr as *const _ as *const SOCKADDR,
            size_of::<SOCKADDR_IN>() as i32,
        )
    })?;

    check_wsa_zero(unsafe { listen(listener.0, 1) })?;

    // Read back the assigned port.
    let mut local_addr: SOCKADDR_IN = unsafe { zeroed() };
    let mut local_addr_len = size_of::<SOCKADDR_IN>() as i32;
    check_wsa_zero(unsafe {
        getsockname(
            listener.0,
            &mut local_addr as *mut _ as *mut SOCKADDR,
            &mut local_addr_len,
        )
    })?;
    let server_port = u16::from_be(local_addr.sin_port);

    let iocp = IocpGuard(unsafe { CreateIoCompletionPort(INVALID_HANDLE_VALUE, None, 0, 0)? });

    associate_socket_with_iocp(iocp.0, listener.0, 1)?;
    associate_socket_with_iocp(iocp.0, accepted.0, 2)?;
    associate_socket_with_iocp(iocp.0, client.0, 3)?;

    const WSAID_ACCEPTEX: GUID = GUID::from_u128(0xb5367df1cbac11cf95ca00805f48a192);
    const WSAID_CONNECTEX: GUID = GUID::from_u128(0x25a207b9ddf346608ee976e58c74063e);

    let accept_ex: AcceptExFn = get_extension_function(listener.0, WSAID_ACCEPTEX)?;
    let connect_ex: ConnectExFn = get_extension_function(client.0, WSAID_CONNECTEX)?;

    let mut accept_ctx = Box::new(OverlappedContext::new(Operation::Accept, ACCEPT_ADDR_SPACE));
    let mut connect_ctx = Box::new(OverlappedContext::new(Operation::Connect, 0));
    let mut client_send_ctx =
        Box::new(OverlappedContext::new(Operation::ClientSend, payload.len()));
    let mut client_recv_ctx =
        Box::new(OverlappedContext::new(Operation::ClientRecv, payload.len()));
    let mut server_recv_ctx =
        Box::new(OverlappedContext::new(Operation::ServerRecv, payload.len()));
    let mut server_send_ctx =
        Box::new(OverlappedContext::new(Operation::ServerSend, payload.len()));

    client_send_ctx.buffer.copy_from_slice(payload);
    client_send_ctx.wsabuf.len = payload.len() as u32;

    // Post AcceptEx. It needs a dedicated, unbound, unconnected accept socket.
    let mut accept_sync_bytes = 0u32;
    check_wsa_overlapped_bool(unsafe {
        accept_ex(
            listener.0,
            accepted.0,
            accept_ctx.buffer.as_mut_ptr() as *mut std::ffi::c_void,
            0,
            (size_of::<SOCKADDR_IN>() + 16) as u32,
            (size_of::<SOCKADDR_IN>() + 16) as u32,
            &mut accept_sync_bytes,
            &mut accept_ctx.overlapped,
        )
    })?;

    // ConnectEx requires the client socket to be bound first, but to a LOCAL address.
    let client_bind_addr = sockaddr_loopback(0);
    check_wsa_zero(unsafe {
        bind(
            client.0,
            &client_bind_addr as *const _ as *const SOCKADDR,
            size_of::<SOCKADDR_IN>() as i32,
        )
    })?;

    let server_addr = sockaddr_loopback(server_port);
    let mut connect_sync_bytes = 0u32;
    check_wsa_overlapped_bool(unsafe {
        connect_ex(
            client.0,
            &server_addr as *const _ as *const SOCKADDR,
            size_of::<SOCKADDR_IN>() as i32,
            null(),
            0,
            &mut connect_sync_bytes,
            &mut connect_ctx.overlapped,
        )
    })?;

    loop {
        let mut bytes_transferred = 0u32;
        let mut completion_key = 0usize;
        let mut overlapped_ptr: *mut OVERLAPPED = null_mut();

        let gqcs = unsafe {
            GetQueuedCompletionStatus(
                iocp.0,
                &mut bytes_transferred,
                &mut completion_key,
                &mut overlapped_ptr,
                IOCP_TIMEOUT_MS,
            )
        };

        if let Err(_e) = gqcs {
            // For GetQueuedCompletionStatus, use GetLastError, not WSAGetLastError.
            let err = unsafe { GetLastError() };

            if overlapped_ptr.is_null() {
                if err.0 == WAIT_TIMEOUT.0 {
                    return Err(win32_error(err.0));
                }
                return Err(win32_error(err.0));
            }

            // A failed I/O completed and was dequeued.
            return Err(win32_error(err.0));
        }

        if overlapped_ptr.is_null() {
            return Err(last_win32_error());
        }

        // SAFETY: OVERLAPPED is the first field in OverlappedContext and the Box allocation is stable.
        let ctx = unsafe { &mut *(overlapped_ptr as *mut OverlappedContext) };

        match ctx.op {
            Operation::Accept => {
                // Make accepted socket inherit the listener context.
                check_wsa_zero(unsafe {
                    setsockopt(
                        accepted.0,
                        SOL_SOCKET as i32,
                        SO_UPDATE_ACCEPT_CONTEXT as i32,
                        Some(socket_opt_bytes(&listener.0)),
                    )
                })?;

                let mut flags = 0u32;
                check_wsa_overlapped_i32(unsafe {
                    WSARecv(
                        accepted.0,
                        &[server_recv_ctx.wsabuf],
                        None,
                        &mut flags,
                        Some(&mut server_recv_ctx.overlapped),
                        None,
                    )
                })?;
            }

            Operation::Connect => {
                // Finalize ConnectEx socket state.
                check_wsa_zero(unsafe {
                    setsockopt(
                        client.0,
                        SOL_SOCKET as i32,
                        SO_UPDATE_CONNECT_CONTEXT as i32,
                        None,
                    )
                })?;

                check_wsa_overlapped_i32(unsafe {
                    WSASend(
                        client.0,
                        &[client_send_ctx.wsabuf],
                        None,
                        0,
                        Some(&mut client_send_ctx.overlapped),
                        None,
                    )
                })?;
            }

            Operation::ClientSend => {
                let mut flags = 0u32;
                check_wsa_overlapped_i32(unsafe {
                    WSARecv(
                        client.0,
                        &[client_recv_ctx.wsabuf],
                        None,
                        &mut flags,
                        Some(&mut client_recv_ctx.overlapped),
                        None,
                    )
                })?;
            }

            Operation::ServerRecv => {
                if bytes_transferred == 0 {
                    return Ok(Vec::new());
                }

                let n = bytes_transferred as usize;
                server_send_ctx.buffer[..n].copy_from_slice(&server_recv_ctx.buffer[..n]);
                server_send_ctx.wsabuf.len = bytes_transferred;

                check_wsa_overlapped_i32(unsafe {
                    WSASend(
                        accepted.0,
                        &[server_send_ctx.wsabuf],
                        None,
                        0,
                        Some(&mut server_send_ctx.overlapped),
                        None,
                    )
                })?;
            }

            Operation::ServerSend => {
                // Nothing else to do here for the single-echo demo.
                let _ = completion_key;
            }

            Operation::ClientRecv => {
                if bytes_transferred == 0 {
                    return Ok(Vec::new());
                }
                return Ok(client_recv_ctx.buffer[..bytes_transferred as usize].to_vec());
            }
        }
    }
}
