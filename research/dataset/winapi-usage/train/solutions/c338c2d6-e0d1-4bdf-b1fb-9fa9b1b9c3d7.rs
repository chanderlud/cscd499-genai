use windows::Win32::Foundation::HANDLE;
use windows::Win32::Security::{GetTokenInformation, TOKEN_INFORMATION_CLASS};

fn call_get_token_information() -> windows::core::HRESULT {
    let token_handle = HANDLE(std::ptr::null_mut());
    let token_class = TOKEN_INFORMATION_CLASS(1);

    let result =
        unsafe { GetTokenInformation(token_handle, token_class, None, 0, std::ptr::null_mut()) };

    match result {
        Ok(()) => windows::core::HRESULT(0),
        Err(e) => e.code(),
    }
}
