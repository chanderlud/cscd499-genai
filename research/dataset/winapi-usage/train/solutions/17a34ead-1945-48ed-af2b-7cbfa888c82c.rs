use windows::core::{Error, Result};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::JobObjects::AssignProcessToJobObject;

fn call_assign_process_to_job_object() -> Result<Result<()>> {
    let hjob = HANDLE::default();
    let hprocess = HANDLE::default();
    Ok(unsafe { AssignProcessToJobObject(hjob, hprocess) })
}
