use windows::core::{HRESULT, PCWSTR};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::AddLogContainer;

fn call_add_log_container() -> HRESULT {
    unsafe {
        match AddLogContainer(HANDLE::default(), None, PCWSTR::null(), None) {
            Ok(()) => HRESULT(0),
            Err(e) => e.code(),
        }
    }
}
