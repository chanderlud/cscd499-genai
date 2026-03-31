use windows::core::{Error, Result};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::JobObjects::CreateJobObjectW;

fn call_create_job_object_w() -> Result<HANDLE> {
    // SAFETY: CreateJobObjectW is safe to call with valid parameters.
    // We pass None for default security attributes and a valid wide string for the job object name.
    unsafe { CreateJobObjectW(None, windows::core::w!("MyJobObject")) }
}
