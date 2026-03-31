use windows::Win32::System::WinRT::CoDecodeProxy;

fn call_co_decode_proxy() -> windows::Win32::Foundation::WIN32_ERROR {
    match unsafe { CoDecodeProxy(0, 0) } {
        Ok(_) => windows::Win32::Foundation::WIN32_ERROR(0),
        Err(e) => windows::Win32::Foundation::WIN32_ERROR::from_error(&e).unwrap_or_default(),
    }
}
