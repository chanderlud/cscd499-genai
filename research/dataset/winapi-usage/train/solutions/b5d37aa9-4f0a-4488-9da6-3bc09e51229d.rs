#![deny(warnings)]

use windows::core::w;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::JobObjects::CreateJobObjectW;

#[allow(dead_code)]
fn call_create_job_object_w() -> WIN32_ERROR {
    match unsafe { CreateJobObjectW(None, w!("TestJob")) } {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
