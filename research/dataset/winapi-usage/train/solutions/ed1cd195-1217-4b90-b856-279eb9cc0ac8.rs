use windows::core::{Error, Result, HRESULT};
use windows::Win32::System::JobObjects::CreateJobObjectA;

fn call_create_job_object_a() -> HRESULT {
    unsafe {
        CreateJobObjectA(None, windows::core::s!("MyJobObject"))
            .map(|_| HRESULT(0))
            .map_err(|e| e.code())
            .unwrap_or_else(|h| h)
    }
}
