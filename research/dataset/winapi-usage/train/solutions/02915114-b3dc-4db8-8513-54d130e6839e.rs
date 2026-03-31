use windows::core::{Error, Result};
use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::System::JobObjects::AssignProcessToJobObject;

#[allow(dead_code)]
fn call_assign_process_to_job_object() -> WIN32_ERROR {
    match unsafe { AssignProcessToJobObject(HANDLE::default(), HANDLE::default()) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
