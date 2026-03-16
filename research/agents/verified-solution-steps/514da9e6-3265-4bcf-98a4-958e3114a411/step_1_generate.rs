use windows::core::{Result, Error, HRESULT, PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE, LocalFree, HLOCAL};
use windows::Win32::Security::{
    GetTokenInformation, TokenUser, ConvertSidToStringSidW, TOKEN_QUERY,
    TOKEN_USER, SID_AND_ATTRIBUTES,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

pub fn current_user_sid_string() -> Result<String> {
    // Open the current process token
    let mut token_handle = HANDLE::default();
    unsafe {
        // SAFETY: We're passing valid pointers to GetCurrentProcess and &mut token_handle
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle)
            .ok()
            .map_err(|_| Error::from_win32())?;
    }

    // Ensure we close the token handle when we're done
    let token_guard = TokenHandle(token_handle);

    // First call to GetTokenInformation to get required buffer size
    let mut return_length = 0u32;
    unsafe {
        // SAFETY: We're passing a null buffer and 0 size to get required length
        GetTokenInformation(
            token_guard.0,
            TokenUser,
            None,
            0,
            &mut return_length,
        );
    }

    // Allocate buffer for token information
    let mut buffer = vec![0u8; return_length as usize];
    let token_user = buffer.as_mut_ptr() as *mut TOKEN_USER;

    // Second call to actually get the token information
    unsafe {
        // SAFETY: We've allocated a buffer of the correct size and are passing valid pointers
        GetTokenInformation(
            token_guard.0,
            TokenUser,
            Some(token_user as *mut _),
            return_length,
            &mut return_length,
        )
        .ok()
        .map_err(|_| Error::from_win32())?;
    }

    // Extract the SID from TOKEN_USER
    let sid = unsafe {
        // SAFETY: We've successfully retrieved token information, so the SID pointer is valid
        (*token_user).User.Sid
    };

    // Convert SID to string
    let mut sid_string_ptr = PWSTR::null();
    unsafe {
        // SAFETY: We're passing a valid SID and a valid pointer to receive the string
        ConvertSidToStringSidW(sid, &mut sid_string_ptr)
            .ok()
            .map_err(|_| Error::from_win32())?;
    }

    // Convert the wide string to a Rust String
    let sid_string = unsafe {
        // SAFETY: ConvertSidToStringSidW allocated a valid wide string for us
        let len = (0..).take_while(|&i| *sid_string_ptr.0.add(i) != 0).count();
        let wide_slice = std::slice::from_raw_parts(sid_string_ptr.0, len);
        String::from_utf16_lossy(wide_slice)
    };

    // Free the string allocated by ConvertSidToStringSidW
    unsafe {
        // SAFETY: We're freeing the string allocated by ConvertSidToStringSidW
        LocalFree(HLOCAL(sid_string_ptr.0 as _));
    }

    Ok(sid_string)
}

// Helper struct to ensure token handle is closed
struct TokenHandle(HANDLE);

impl Drop for TokenHandle {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: We're closing a valid handle that we opened
            let _ = CloseHandle(self.0);
        }
    }
}