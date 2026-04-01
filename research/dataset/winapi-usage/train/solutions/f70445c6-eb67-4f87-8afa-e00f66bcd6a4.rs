use windows::core::Result;
use windows::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileA, FILE_ATTRIBUTE_NORMAL, FILE_CREATION_DISPOSITION, FILE_FLAGS_AND_ATTRIBUTES,
    FILE_SHARE_MODE, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_ALWAYS,
};

fn call_create_file_a() -> Result<HANDLE> {
    let path = "C:\\temp\\test.txt";
    let filename = windows::core::PCSTR(path.as_ptr() as *const u8);

    // CreateFileA is marked unsafe in the windows crate
    unsafe {
        CreateFileA(
            filename,
            GENERIC_READ.0 | GENERIC_WRITE.0,
            FILE_SHARE_MODE(FILE_SHARE_READ.0 | FILE_SHARE_WRITE.0),
            None,
            FILE_CREATION_DISPOSITION(OPEN_ALWAYS.0),
            FILE_FLAGS_AND_ATTRIBUTES(FILE_ATTRIBUTE_NORMAL.0),
            None,
        )
    }
}
