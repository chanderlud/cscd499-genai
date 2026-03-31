use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::JobObjects::AssignProcessToJobObject;

fn call_assign_process_to_job_object() -> HRESULT {
    unsafe {
        match AssignProcessToJobObject(HANDLE::default(), HANDLE::default()) {
            Ok(()) => HRESULT::default(),
            Err(e) => e.code(),
        }
    }
}
