use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::JobObjects::{CreateJobSet, JOB_SET_ARRAY};

fn call_create_job_set() -> Result<WIN32_ERROR> {
    let job_set: [JOB_SET_ARRAY; 0] = [];
    let result = unsafe { CreateJobSet(&job_set, 0) };

    if result.as_bool() {
        Ok(WIN32_ERROR(0))
    } else {
        Err(Error::from_thread())
    }
}
