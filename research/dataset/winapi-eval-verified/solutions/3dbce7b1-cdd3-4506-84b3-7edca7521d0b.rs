use windows::Win32::Foundation::{
    CloseHandle, ERROR_PIPE_CONNECTED, GENERIC_WRITE, HANDLE, HLOCAL, INVALID_HANDLE_VALUE,
    LocalFree,
};
use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
use windows::Win32::Security::{
    GetTokenInformation, RevertToSelf, TOKEN_QUERY, TOKEN_USER, TokenUser,
};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_MODE, OPEN_EXISTING, PIPE_ACCESS_DUPLEX,
    ReadFile, WriteFile,
};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, ImpersonateNamedPipeClient, PIPE_READMODE_BYTE,
    PIPE_TYPE_BYTE, PIPE_WAIT,
};
use windows::Win32::System::Threading::{GetCurrentThread, OpenThreadToken};
use windows::core::{Error, HRESULT, PCWSTR, PWSTR, Result};

fn wide_null(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn named_pipe_impersonated_sid(pipe_name: &str) -> Result<String> {
    if pipe_name.is_empty() || !pipe_name.starts_with(r"\\.\pipe\") {
        return Err(Error::new(
            HRESULT(0x80070057u32 as i32), // E_INVALIDARG / ERROR_INVALID_PARAMETER-ish
            "pipe_name must start with \\\\.\\pipe\\ and not be empty",
        ));
    }

    let pipe_name_wide = wide_null(pipe_name);

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

    let pipe_name_owned = pipe_name.to_owned();
    let client_thread = std::thread::spawn(move || -> Result<()> {
        let pipe_name_wide = wide_null(&pipe_name_owned);

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

        let data = [0u8; 1];
        let mut bytes_written = 0;
        unsafe { WriteFile(client_handle, Some(&data), Some(&mut bytes_written), None) }?;

        unsafe { CloseHandle(client_handle) }?;
        Ok(())
    });

    match unsafe { ConnectNamedPipe(pipe_handle, None) } {
        Ok(()) => {}
        Err(e) if e.code() == HRESULT::from_win32(ERROR_PIPE_CONNECTED.0) => {}
        Err(e) => {
            let _ = unsafe { CloseHandle(pipe_handle) };
            let _ = client_thread.join();
            return Err(e);
        }
    }

    // REQUIRED: read at least one byte before impersonating
    let mut buf = [0u8; 1];
    let mut bytes_read = 0;
    unsafe { ReadFile(pipe_handle, Some(&mut buf), Some(&mut bytes_read), None) }?;

    unsafe { ImpersonateNamedPipeClient(pipe_handle) }?;

    let result = (|| -> Result<String> {
        let mut token_handle = HANDLE::default();
        unsafe { OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, false, &mut token_handle) }?;

        let mut return_length = 0;
        let first =
            unsafe { GetTokenInformation(token_handle, TokenUser, None, 0, &mut return_length) };

        if first.is_err() && return_length == 0 {
            unsafe { CloseHandle(token_handle) }?;
            return Err(first.err().unwrap());
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

        let token_user = buffer.as_ptr() as *const TOKEN_USER;
        let sid = unsafe { (*token_user).User.Sid };

        let mut sid_string_ptr = PWSTR::null();
        unsafe { ConvertSidToStringSidW(sid, &mut sid_string_ptr) }?;

        let sid_string = unsafe { sid_string_ptr.to_string() }?;

        unsafe { LocalFree(Some(HLOCAL(sid_string_ptr.0 as *mut _))) };
        unsafe { CloseHandle(token_handle) }?;

        Ok(sid_string)
    })();

    let _ = unsafe { RevertToSelf() };
    let _ = unsafe { CloseHandle(pipe_handle) };
    let _ = client_thread.join();

    result
}
