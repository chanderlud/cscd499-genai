use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Wmi::{MI_Application, MI_Application_InitializeV1};

fn call_mi__application__initialize_v1() -> WIN32_ERROR {
    let mut app: MI_Application = unsafe { std::mem::zeroed() };
    let result = unsafe { MI_Application_InitializeV1(0, None, None, &mut app) };
    WIN32_ERROR(result.0 as u32)
}
