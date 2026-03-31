use windows::Win32::Foundation::S_OK;
use windows::Win32::System::WinRT::CoDecodeProxy;

fn call_co_decode_proxy() -> windows::core::HRESULT {
    match unsafe { CoDecodeProxy(0, 0) } {
        Ok(_) => S_OK,
        Err(e) => e.code(),
    }
}
