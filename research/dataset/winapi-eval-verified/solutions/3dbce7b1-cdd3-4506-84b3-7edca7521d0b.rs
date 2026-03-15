use windows::core::{Result, Error, PWSTR, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, LocalFree, INVALID_HANDLE_VALUE, HANDLE, HLOCAL, GENERIC_WRITE};
use windows::Win32::Storage::FileSystem::{CreateFileW, WriteFile, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_MODE, OPEN_EXISTING};
use windows::Win32::System::Pipes::{CreateNamedPipeW, ConnectNamedPipe, ImpersonateNamedPipeClient, PIPE_ACCESS_DUPLEX, PIPE_TYPE_BYTE, PIPE_READMODE_BYTE, PIPE_WAIT};
use windows::Win32::Security::{GetTokenInformation, RevertToSelf, TOKEN_QUERY, TokenUser, TOKEN_USER};
use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
use windows::Win32::System::Threading::{GetCurrentThread, OpenThreadToken};

fn wide_null(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn named_pipe_impersonated_sid(pipe_name: &str) -> Result<String> {
    let pipe_name_wide = wide_null(pipe_name);
    
    // Create named pipe
    let pipe_handle = unsafe {
        CreateNamedPipeW(
            PCWSTR(pipe_name_wide.as_ptr()),
            PIPE_ACCESS_DUPLEX,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
            1,
            4096,
            4096,
            0,
            None,
        )
    };
    
    if pipe_handle == INVALID_HANDLE_VALUE {
        return Err(Error::from_thread());
    }
    
    // Create client thread to connect to pipe
    let pipe_name_owned = pipe_name.to_owned();
    let client_thread = std::thread::spawn(move || -> Result<()> {
        let pipe_name_wide = wide_null(&pipe_name_owned);
        
        // Connect to pipe
        let client_handle = unsafe {
            CreateFileW(
                PCWSTR(pipe_name_wide.as_ptr()),
                GENERIC_WRITE.0,
                FILE_SHARE_MODE(0),
                None,
                OPEN_EXISTING,
                FILE_FLAGS_AND_ATTRIBUTES(0),
                None,
            )
        }?;
        
        // Write to pipe to establish connection
        let data = [0u8; 1];
        let mut bytes_written = 0;
        let result = unsafe {
            WriteFile(
                client_handle,
                Some(&data),
                Some(&mut bytes_written),
                None,
            )
        };
        
        unsafe { CloseHandle(client_handle) }?;
        
        result
    });
    
    // Wait for client to connect
    let result = unsafe { ConnectNamedPipe(pipe_handle, None) };
    if result.is_err() {
        unsafe { CloseHandle(pipe_handle) };
        return Err(Error::from_thread());
    }
    
    // Impersonate client
    unsafe { ImpersonateNamedPipeClient(pipe_handle) }?;
    
    // Get current thread token (with impersonation)
    let mut token_handle = HANDLE::default();
    unsafe { OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, false, &mut token_handle) }?;
    
    // Get token information for TokenUser
    let mut return_length = 0;
    let result = unsafe {
        GetTokenInformation(
            token_handle,
            TokenUser,
            None,
            0,
            &mut return_length,
        )
    };
    
    if result.is_err() && return_length == 0 {
        unsafe {
            CloseHandle(token_handle);
            CloseHandle(pipe_handle);
        }
        return Err(result.err().unwrap());
    }
    
    let mut buffer = vec![0u8; return_length as usize];
    unsafe {
        GetTokenInformation(
            token_handle,
            TokenUser,
            Some(buffer.as_mut_ptr() as *mut _),
            return_length,
            &mut return_length,
        )
    }?;
    
    // Extract SID from TOKEN_USER
    let token_user = buffer.as_ptr() as *const TOKEN_USER;
    let sid = unsafe { (*token_user).User.Sid };
    
    // Convert SID to string
    let mut sid_string_ptr = PWSTR::null();
    unsafe { ConvertSidToStringSidW(sid, &mut sid_string_ptr) }?;
    
    // Convert PWSTR to Rust String
    let sid_string = unsafe { sid_string_ptr.to_string() }?;
    
    // Free memory allocated by ConvertSidToStringSidW
    unsafe { LocalFree(Some(HLOCAL(sid_string_ptr.0 as *mut _))) };
    
    // Revert impersonation
    unsafe { RevertToSelf() }?;
    
    // Close handles
    unsafe {
        CloseHandle(token_handle);
        CloseHandle(pipe_handle);
    }
    
    // Wait for client thread to finish
    let _ = client_thread.join().unwrap();
    
    Ok(sid_string)
}
