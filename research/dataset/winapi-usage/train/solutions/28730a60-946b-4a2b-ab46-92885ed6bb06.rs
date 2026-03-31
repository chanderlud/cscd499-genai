use windows::core::PCWSTR;
use windows::core::{Error, Result};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::AddLogContainerSet;

fn call_add_log_container_set() -> Result<Result<()>> {
    let hlog = HANDLE::default();
    let pcbcontainer = None;
    let rgwszcontainerpath: &[PCWSTR] = &[];
    let preserved = None;

    // SAFETY: Calling AddLogContainerSet with default/null parameters.
    // The API returns a Result capturing any HRESULT/Win32 error.
    let res = unsafe { AddLogContainerSet(hlog, pcbcontainer, rgwszcontainerpath, preserved) };
    Ok(res)
}
