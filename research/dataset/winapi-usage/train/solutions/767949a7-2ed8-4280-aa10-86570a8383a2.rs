use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::System::ProcessStatus::EmptyWorkingSet;

fn call_empty_working_set() -> WIN32_ERROR {
    match unsafe { EmptyWorkingSet(HANDLE::default()) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
