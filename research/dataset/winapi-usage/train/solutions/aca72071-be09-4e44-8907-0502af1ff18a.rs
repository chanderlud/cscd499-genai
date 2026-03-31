use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Memory::{CreateFileMappingW, PAGE_READWRITE};

fn call_create_file_mapping_w() -> HRESULT {
    unsafe {
        CreateFileMappingW(
            HANDLE::default(),
            None,
            PAGE_READWRITE,
            0,
            0,
            windows::core::PCWSTR::null(),
        )
        .map(|_| HRESULT::default())
        .unwrap_or_else(|e: Error| e.code())
    }
}
