use windows::core::{Error, Result};
use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::Security::{GetTokenInformation, TOKEN_INFORMATION_CLASS};

fn call_get_token_information() -> windows::Win32::Foundation::WIN32_ERROR {
    let token_handle = HANDLE(std::ptr::null_mut());
    let info_class = TOKEN_INFORMATION_CLASS(1);
    let mut buffer: [u8; 256] = [0; 256];
    let mut return_length: u32 = 0;

    let result = unsafe {
        GetTokenInformation(
            token_handle,
            info_class,
            Some(buffer.as_mut_ptr() as *mut core::ffi::c_void),
            buffer.len() as u32,
            &mut return_length,
        )
    };

    match result {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
