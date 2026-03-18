use std::ffi::CString;
use std::ptr;
use std::thread;

use windows::core::Error;
use windows::Win32::Networking::WinSock::{
    accept, bind, closesocket, connect, getsockname, inet_addr, listen, recv, send, shutdown,
    socket, WSACleanup, WSAStartup, AF_INET, IN_ADDR, IPPROTO_TCP, SD_SEND, SEND_RECV_FLAGS,
    SOCKADDR, SOCKADDR_IN, SOCKET_ERROR, SOCK_STREAM, WINSOCK_SOCKET_TYPE, WSADATA,
};

fn winsock_error() -> Error {
    Error::from_thread()
}

pub fn winsock_tcp_reverse_echo(payload: &[u8]) -> std::io::Result<Vec<u8>> {
    // Initialize Winsock
    let mut wsa_data = WSADATA::default();
    let result = unsafe { WSAStartup(0x0202, &mut wsa_data) };
    if result != 0 {
        return Err(winsock_error().into());
    }

    // Ensure WSACleanup is called on exit
    struct WsaGuard;
    impl Drop for WsaGuard {
        fn drop(&mut self) {
            unsafe { WSACleanup() };
        }
    }
    let _guard = WsaGuard;

    // Create server socket
    let server_socket = unsafe {
        socket(
            AF_INET.0.into(),
            WINSOCK_SOCKET_TYPE(SOCK_STREAM.0),
            IPPROTO_TCP.0.into(),
        )
    }?;

    // Bind to 127.0.0.1:0 (ephemeral port)
    let addr = CString::new("127.0.0.1").unwrap();
    let mut server_addr = SOCKADDR_IN {
        sin_family: AF_INET,
        sin_port: 0, // Let system assign port
        sin_addr: IN_ADDR {
            S_un: windows::Win32::Networking::WinSock::IN_ADDR_0 {
                S_addr: unsafe {
                    inet_addr(windows::core::PCSTR::from_raw(addr.as_ptr() as *const u8))
                },
            },
        },
        sin_zero: [0; 8],
    };

    let result = unsafe {
        bind(
            server_socket,
            &server_addr as *const _ as *const SOCKADDR,
            std::mem::size_of::<SOCKADDR_IN>() as i32,
        )
    };
    if result == SOCKET_ERROR {
        unsafe { closesocket(server_socket) };
        return Err(winsock_error().into());
    }

    // Get assigned port
    let mut addr_len = std::mem::size_of::<SOCKADDR_IN>() as i32;
    let result = unsafe {
        getsockname(
            server_socket,
            &mut server_addr as *mut _ as *mut SOCKADDR,
            &mut addr_len,
        )
    };
    if result == SOCKET_ERROR {
        unsafe { closesocket(server_socket) };
        return Err(winsock_error().into());
    }
    let port = server_addr.sin_port;

    // Listen for connections
    let result = unsafe { listen(server_socket, 1) };
    if result == SOCKET_ERROR {
        unsafe { closesocket(server_socket) };
        return Err(winsock_error().into());
    }

    // Spawn server thread
    let server_handle = thread::spawn(move || -> std::io::Result<Vec<u8>> {
        // Accept client connection
        let client_socket =
            unsafe { accept(server_socket, Some(ptr::null_mut()), Some(ptr::null_mut())) }?;

        // Receive data from client
        let mut received = Vec::new();
        let mut buf = [0u8; 1024];
        loop {
            let bytes_received = unsafe { recv(client_socket, &mut buf[..], SEND_RECV_FLAGS(0)) };
            if bytes_received == SOCKET_ERROR {
                let err = winsock_error();
                unsafe {
                    closesocket(client_socket);
                    closesocket(server_socket);
                }
                return Err(err.into());
            }
            if bytes_received == 0 {
                break; // Connection closed
            }
            received.extend_from_slice(&buf[..bytes_received as usize]);
        }

        // Reverse the received data
        received.reverse();

        // Send reversed data back
        let mut total_sent = 0;
        while total_sent < received.len() {
            let bytes_sent =
                unsafe { send(client_socket, &received[total_sent..], SEND_RECV_FLAGS(0)) };
            if bytes_sent == SOCKET_ERROR {
                let err = winsock_error();
                unsafe {
                    closesocket(client_socket);
                    closesocket(server_socket);
                }
                return Err(err.into());
            }
            total_sent += bytes_sent as usize;
        }

        // Cleanup
        unsafe {
            closesocket(client_socket);
            closesocket(server_socket);
        }

        Ok(received)
    });

    // Client operations
    let client_result = (|| -> std::io::Result<Vec<u8>> {
        // Create client socket
        let client_socket = unsafe {
            socket(
                AF_INET.0.into(),
                WINSOCK_SOCKET_TYPE(SOCK_STREAM.0),
                IPPROTO_TCP.0.into(),
            )
        }?;

        // Connect to server
        let addr = CString::new("127.0.0.1").unwrap();
        let server_addr = SOCKADDR_IN {
            sin_family: AF_INET,
            sin_port: port,
            sin_addr: IN_ADDR {
                S_un: windows::Win32::Networking::WinSock::IN_ADDR_0 {
                    S_addr: unsafe {
                        inet_addr(windows::core::PCSTR::from_raw(addr.as_ptr() as *const u8))
                    },
                },
            },
            sin_zero: [0; 8],
        };

        let result = unsafe {
            connect(
                client_socket,
                &server_addr as *const _ as *const SOCKADDR,
                std::mem::size_of::<SOCKADDR_IN>() as i32,
            )
        };
        if result == SOCKET_ERROR {
            unsafe { closesocket(client_socket) };
            return Err(winsock_error().into());
        }

        // Send payload
        let mut total_sent = 0;
        while total_sent < payload.len() {
            let bytes_sent =
                unsafe { send(client_socket, &payload[total_sent..], SEND_RECV_FLAGS(0)) };
            if bytes_sent == SOCKET_ERROR {
                unsafe { closesocket(client_socket) };
                return Err(winsock_error().into());
            }
            total_sent += bytes_sent as usize;
        }

        // Shutdown send side to signal end of data
        let result = unsafe { shutdown(client_socket, SD_SEND) };
        if result == SOCKET_ERROR {
            unsafe { closesocket(client_socket) };
            return Err(winsock_error().into());
        }

        // Receive response
        let mut response = Vec::new();
        let mut buf = [0u8; 1024];
        loop {
            let bytes_received = unsafe { recv(client_socket, &mut buf[..], SEND_RECV_FLAGS(0)) };
            if bytes_received == SOCKET_ERROR {
                unsafe { closesocket(client_socket) };
                return Err(winsock_error().into());
            }
            if bytes_received == 0 {
                break; // Connection closed
            }
            response.extend_from_slice(&buf[..bytes_received as usize]);
        }

        // Cleanup
        unsafe { closesocket(client_socket) };

        Ok(response)
    })();

    // Wait for server thread and return client result
    let server_result = server_handle
        .join()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Server thread panicked"))?;

    // Check both results
    let client_data = client_result?;
    server_result?;

    // Return the client's received data (which should be the reversed payload)
    Ok(client_data)
}
