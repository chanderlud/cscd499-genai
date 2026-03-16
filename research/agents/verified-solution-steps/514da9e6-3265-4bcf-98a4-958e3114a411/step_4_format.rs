use windows::core::{Error, Result, HRESULT, PWSTR};
use windows::Win32::Foundation::{
    CloseHandle, LocalFree, ERROR_INSUFFICIENT_BUFFER, HANDLE, HLOCAL,
};
use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
use windows::Win32::Security::{GetTokenInformation, TokenUser, TOKEN_QUERY, TOKEN_USER};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

pub fn current_user_sid_string() -> Result<String> {
    // Open the current process token
    let mut token_handle = HANDLE::default();
    unsafe {
        // SAFETY: We're passing valid pointers to GetCurrentProcess and &mut token_handle
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle)?;
    }

    // Ensure we close the token handle when we're done
    let token_guard = TokenHandle(token_handle);

    // First call to GetTokenInformation to get required buffer size
    let mut return_length = 0u32;
    unsafe {
        // SAFETY: We're passing a null buffer and 0 size to get required length
        let result = GetTokenInformation(token_guard.0, TokenUser, None, 0, &mut return_length);

        // The first call is expected to fail with ERROR_INSUFFICIENT_BUFFER
        if let Err(e) = result {
            // Check if it's the expected insufficient buffer error
            if e.code() != HRESULT::from_win32(ERROR_INSUFFICIENT_BUFFER.0) {
                return Err(e);
            }
        } else {
            // If it didn't fail, something is wrong
            return Err(Error::new(
                HRESULT::from_win32(ERROR_INSUFFICIENT_BUFFER.0),
                "Expected ERROR_INSUFFICIENT_BUFFER",
            ));
        }
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
        )?;
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
        ConvertSidToStringSidW(sid, &mut sid_string_ptr)?;
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
        LocalFree(Some(HLOCAL(sid_string_ptr.0 as _)));
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