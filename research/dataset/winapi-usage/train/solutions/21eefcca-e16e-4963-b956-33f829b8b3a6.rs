use windows::core::{Error, Result, HRESULT};
use windows::Win32::System::ProcessStatus::EnumPageFilesA;

fn call_enum_page_files_a() -> HRESULT {
    unsafe {
        match EnumPageFilesA(None, std::ptr::null_mut()) {
            Ok(()) => HRESULT(0),
            Err(e) => e.code(),
        }
    }
}
