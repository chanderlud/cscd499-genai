use windows::core::BOOL;
use windows::core::{Error, Result};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::JobObjects::{CreateJobSet, JOB_SET_ARRAY};

fn call_create_job_set() -> Result<BOOL> {
    let job_set_array = [JOB_SET_ARRAY {
        JobHandle: HANDLE(std::ptr::null_mut()),
        MemberLevel: 0,
        Flags: 0,
    }];

    let result = unsafe { CreateJobSet(&job_set_array, 0) };
    Ok(result)
}
