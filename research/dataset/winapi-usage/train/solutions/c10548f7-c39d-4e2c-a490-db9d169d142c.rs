use windows::core::{Error, HRESULT};
use windows::Win32::Foundation::{INVALID_HANDLE_VALUE, S_OK};
use windows::Win32::System::Memory::{CreateFileMappingA, PAGE_PROTECTION_FLAGS, PAGE_READWRITE};

fn call_create_file_mapping_a() -> HRESULT {
    let result = unsafe {
        CreateFileMappingA(
            INVALID_HANDLE_VALUE,
            None,
            PAGE_READWRITE,
            0,
            0,
            windows::core::PCSTR::null(),
        )
    };

    match result {
        Ok(handle) => {
            if handle.0 != INVALID_HANDLE_VALUE.0 {
                S_OK
            } else {
                Error::from_thread().code()
            }
        }
        Err(_) => Error::from_thread().code(),
    }
}
