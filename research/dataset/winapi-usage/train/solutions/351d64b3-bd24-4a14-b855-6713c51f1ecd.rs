use windows::core::Result;
use windows::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileA, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, FILE_CREATION_DISPOSITION,
    FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_MODE, FILE_SHARE_READ, FILE_SHARE_WRITE,
};

fn call_create_file_a() -> windows::core::HRESULT {
    let filename = "C:\\temp\\test.txt";

    let result: Result<HANDLE> = unsafe {
        CreateFileA(
            windows::core::PCSTR(filename.as_ptr() as *const u8),
            GENERIC_READ.0 | GENERIC_WRITE.0,
            FILE_SHARE_MODE(FILE_SHARE_READ.0 | FILE_SHARE_WRITE.0),
            None,
            FILE_CREATION_DISPOSITION(CREATE_ALWAYS.0),
            FILE_FLAGS_AND_ATTRIBUTES(FILE_ATTRIBUTE_NORMAL.0),
            None,
        )
    };

    match result {
        Ok(_) => windows::core::HRESULT::from_win32(0),
        Err(e) => e.code(),
    }
}
