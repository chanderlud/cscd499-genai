use windows::core::HRESULT;
use windows::Win32::System::Power::CanUserWritePwrScheme;

fn call_can_user_write_pwr_scheme() -> HRESULT {
    unsafe {
        CanUserWritePwrScheme();
        HRESULT(0)
    }
}
