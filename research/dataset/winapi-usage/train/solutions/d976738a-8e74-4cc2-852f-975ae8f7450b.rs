use windows::core::{Error, Result};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::ProcessStatus::EmptyWorkingSet;

#[allow(dead_code)]
fn call_empty_working_set() -> Result<Result<()>> {
    let handle = HANDLE::default();
    Ok(unsafe { EmptyWorkingSet(handle) })
}
