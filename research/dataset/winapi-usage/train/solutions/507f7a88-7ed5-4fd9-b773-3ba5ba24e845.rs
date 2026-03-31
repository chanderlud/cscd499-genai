use windows::core::PCWSTR;
use windows::core::{Error, Result};
use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::Storage::FileSystem::AddLogContainerSet;

fn call_add_log_container_set() -> WIN32_ERROR {
    let hlog = HANDLE(std::ptr::null_mut());
    let pcbcontainer = None;
    let rgwszcontainerpath: &[PCWSTR] = &[];
    let preserved = None;

    // SAFETY: We pass valid null/empty parameters as required by the API signature.
    // The function will fail with an error code, which we capture and return.
    match unsafe { AddLogContainerSet(hlog, pcbcontainer, rgwszcontainerpath, preserved) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
