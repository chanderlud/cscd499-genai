use windows::Win32::Foundation::HANDLE;
use windows::Win32::Security::{GetTokenInformation, TOKEN_INFORMATION_CLASS};

fn call_get_token_information() -> windows::core::Result<windows::core::Result<()>> {
    let token_handle = HANDLE::default();
    let token_class = TOKEN_INFORMATION_CLASS(1);
    let mut buffer = [0u8; 256];
    let mut return_length = 0u32;

    Ok(unsafe {
        GetTokenInformation(
            token_handle,
            token_class,
            Some(buffer.as_mut_ptr() as *mut core::ffi::c_void),
            buffer.len() as u32,
            &mut return_length,
        )
    })
}
