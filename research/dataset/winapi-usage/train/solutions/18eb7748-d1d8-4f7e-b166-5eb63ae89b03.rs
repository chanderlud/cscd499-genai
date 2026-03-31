use windows::core::{Error, Result, HRESULT};
use windows::Win32::System::Wmi::{MI_Application, MI_Application_InitializeV1};

fn call_mi__application__initialize_v1() -> windows::core::HRESULT {
    let mut app = std::mem::MaybeUninit::<MI_Application>::uninit();
    let result = unsafe { MI_Application_InitializeV1(0, None, None, app.as_mut_ptr()) };
    HRESULT(result.0)
}
