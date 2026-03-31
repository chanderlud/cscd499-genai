#![deny(warnings)]

use windows::core::HRESULT;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::AddLogContainerSet;

#[allow(dead_code)]
fn call_add_log_container_set() -> HRESULT {
    unsafe {
        AddLogContainerSet(HANDLE::default(), None, &[], None)
            .map(|_| HRESULT(0))
            .unwrap_or_else(|e| e.code())
    }
}
