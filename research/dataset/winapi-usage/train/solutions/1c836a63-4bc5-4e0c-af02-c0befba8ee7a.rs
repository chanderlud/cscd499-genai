use windows::core::Result;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::JobObjects::CreateJobObjectA;

fn call_create_job_object_a() -> Result<HANDLE> {
    // SAFETY: CreateJobObjectA is a Win32 API. We pass None for security attributes
    // and a valid, null-terminated ANSI string for the job object name.
    unsafe { CreateJobObjectA(None, windows::core::PCSTR(b"MyJobObject\0".as_ptr())) }
}
