use windows::core::HRESULT;
use windows::Win32::System::Ole::CreateOleAdviseHolder;

fn call_create_ole_advise_holder() -> windows::core::HRESULT {
    unsafe {
        CreateOleAdviseHolder()
            .map(|_| HRESULT::from_win32(0))
            .unwrap_or_else(|hr| hr.into())
    }
}
