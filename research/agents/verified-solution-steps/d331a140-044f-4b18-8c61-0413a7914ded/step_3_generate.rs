use std::sync::{Arc, Barrier};
use std::thread;
use windows::core::{Result, Error, PCWSTR, HRESULT};
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE, ERROR_PIPE_CONNECTED};
use windows::Win32::Storage::FileSystem::{CreateFileW, FILE_GENERIC_READ, FILE_GENERIC_WRITE, OPEN_EXISTING, PIPE_ACCESS_DUPLEX};
use windows::Win32::System::Pipes::{CreateNamedPipeW, ConnectNamedPipe, DisconnectNamedPipe, PIPE_READMODE_BYTE, PIPE_TYPE_BYTE, PIPE_WAIT};
use windows::Win32::Storage::FileSystem::{ReadFile, WriteFile};

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

fn create_named_pipe_instances(pipe_name: &str, count: u32) -> Result<Vec<HANDLE>> {
    let wide_name = wide_null(pipe_name);
    let mut handles = Vec::with_capacity(count as usize);
    
    for _ in 0..count {
        let handle = unsafe {
            CreateNamedPipeW(
                PCWSTR(wide_name.as_ptr()),
                PIPE_ACCESS_DUPLEX,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                count,
                4096,
                4096,
                0,
                None,
            )
        };
        
        if handle == INVALID_HANDLE_VALUE {
            // Close any previously created handles before returning error
            for h in handles {
                unsafe { CloseHandle(h)? };
            }
            return Err(Error::from_thread());
        }
        
        handles.push(handle);
    }
    
    Ok(handles)
}

fn server_thread(handle_raw: isize, barrier: Arc<Barrier>) -> Result<()> {
    let handle = HANDLE(handle_raw as _);
    
    // Wait for all threads to be ready
    barrier.wait();
    
    // Connect to the client
    let connected = unsafe { ConnectNamedPipe(handle, None) };
    if let Err(e) = connected {
        // ERROR_PIPE_CONNECTED means client already connected, which is fine
        if e.code() != HRESULT::from_win32(ERROR_PIPE_CONNECTED.0) {
            unsafe { CloseHandle(handle)? };
            return Err(e);
        }
    }
    
    // Read client index (4 bytes)
    let mut buffer = [0u8; 4];
    let mut bytes_read = 0u32;
    let read_result = unsafe {
        ReadFile(
            handle,
            Some(&mut buffer),
            Some(&mut bytes_read),
            None,
        )
    };
    
    if let Err(e) = read_result {
        unsafe { DisconnectNamedPipe(handle)? };
        unsafe { CloseHandle(handle)? };
        return Err(e);
    }
    
    let client_index = u32::from_le_bytes(buffer);
    
    // Calculate response: i * 3 + 1
    let response = client_index * 3 + 1;
    let response_bytes = response.to_le_bytes();
    
    // Write response
    let mut bytes_written = 0u32;
    let write_result = unsafe {
        WriteFile(
            handle,
            Some(&response_bytes),
            Some(&mut bytes_written),
            None,
        )
    };
    
    if let Err(e) = write_result {
        unsafe { DisconnectNamedPipe(handle)? };
        unsafe { CloseHandle(handle)? };
        return Err(e);
    }
    
    // Cleanup
    unsafe { DisconnectNamedPipe(handle)? };
    unsafe { CloseHandle(handle)? };
    
    Ok(())
}

fn client_thread(pipe_name: &str, index: u32, barrier: Arc<Barrier>) -> Result<u32> {
    let wide_name = wide_null(pipe_name);
    
    // Wait for all threads to be ready
    barrier.wait();
    
    // Connect to the pipe
    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide_name.as_ptr()),
            (FILE_GENERIC_READ | FILE_GENERIC_WRITE).0,
            windows::Win32::Storage::FileSystem::FILE_SHARE_MODE(0),
            None,
            OPEN_EXISTING,
            windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES(0),
            None,
        )
    }?;
    
    // Send index
    let index_bytes = index.to_le_bytes();
    let mut bytes_written = 0u32;
    let write_result = unsafe {
        WriteFile(
            handle,
            Some(&index_bytes),
            Some(&mut bytes_written),
            None,
        )
    };
    
    if let Err(e) = write_result {
        unsafe { CloseHandle(handle)? };
        return Err(e);
    }
    
    // Read response
    let mut buffer = [0u8; 4];
    let mut bytes_read = 0u32;
    let read_result = unsafe {
        ReadFile(
            handle,
            Some(&mut buffer),
            Some(&mut bytes_read),
            None,
        )
    };
    
    if let Err(e) = read_result {
        unsafe { CloseHandle(handle)? };
        return Err(e);
    }
    
    let response = u32::from_le_bytes(buffer);
    
    // Cleanup
    unsafe { CloseHandle(handle)? };
    
    Ok(response)
}

pub fn named_pipe_fanout(pipe_name: &str, client_count: u32) -> std::io::Result<Vec<u32>> {
    let full_pipe_name = format!("\\\\.\\pipe\\{}", pipe_name);
    
    // Create pipe instances
    let handles = create_named_pipe_instances(&full_pipe_name, client_count)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    
    // Create barrier for synchronization
    let barrier = Arc::new(Barrier::new((client_count * 2 + 1) as usize));
    
    // Start server threads
    let mut server_handles = Vec::with_capacity(client_count as usize);
    for handle in handles {
        let barrier_clone = barrier.clone();
        let handle_raw = handle.0 as isize;
        let server_handle = thread::spawn(move || -> Result<()> {
            server_thread(handle_raw, barrier_clone)
        });
        server_handles.push(server_handle);
    }
    
    // Start client threads
    let mut client_handles = Vec::with_capacity(client_count as usize);
    for i in 0..client_count {
        let barrier_clone = barrier.clone();
        let pipe_name_clone = full_pipe_name.clone();
        let client_handle = thread::spawn(move || -> Result<u32> {
            client_thread(&pipe_name_clone, i, barrier_clone)
        });
        client_handles.push(client_handle);
    }
    
    // Wait for all threads to be ready, then release them
    barrier.wait();
    
    // Collect results from client threads
    let mut results = Vec::with_capacity(client_count as usize);
    for handle in client_handles {
        let result = handle.join().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "Client thread panicked")
        })??;
        results.push(result);
    }
    
    // Wait for server threads to finish
    for handle in server_handles {
        handle.join().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "Server thread panicked")
        })??;
    }
    
    Ok(results)
}