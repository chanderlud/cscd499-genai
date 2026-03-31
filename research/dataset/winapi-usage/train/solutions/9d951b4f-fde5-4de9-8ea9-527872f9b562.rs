#![deny(warnings)]

use windows::core::Result;
use windows::Win32::System::Wmi::{MI_Application, MI_Application_InitializeV1, MI_Result};

#[allow(dead_code)]
#[allow(non_snake_case)]
fn call_mi__application__initialize_v1() -> Result<MI_Result> {
    let mut app = MI_Application::default();
    // SAFETY: MI_Application_InitializeV1 requires a valid mutable pointer to an MI_Application struct.
    // We provide a default-initialized instance and pass it safely.
    let result = unsafe { MI_Application_InitializeV1(0, None, None, &mut app) };
    Ok(result)
}
