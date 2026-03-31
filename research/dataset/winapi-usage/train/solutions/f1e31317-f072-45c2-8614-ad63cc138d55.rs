use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::JobObjects::CreateJobObjectA;

fn call_create_job_object_a() -> WIN32_ERROR {
    match unsafe { CreateJobObjectA(None, windows::core::s!("MyJobObject")) } {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
