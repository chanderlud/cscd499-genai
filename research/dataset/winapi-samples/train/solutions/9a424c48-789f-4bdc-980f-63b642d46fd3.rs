use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::core::PCWSTR;
use windows::Win32::Storage::FileSystem::{
    GetFileAttributesExW, GetFileExInfoStandard, WIN32_FILE_ATTRIBUTE_DATA,
};

/// Converts a Rust string to a null-terminated wide string (Vec<u16>)
fn wide_null(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn main() {
    // Example file path - you can change this to any file you want to query
    let file_path = "C:\\Windows\\System32\\kernel32.dll";

    // Convert path to null-terminated wide string
    let path_wide = wide_null(file_path);

    // Create a mutable buffer to receive file attribute information
    let mut file_info = WIN32_FILE_ATTRIBUTE_DATA::default();

    // Call GetFileAttributesExW to query file attributes
    unsafe {
        GetFileAttributesExW(
            PCWSTR::from_raw(path_wide.as_ptr()),
            GetFileExInfoStandard,
            &mut file_info as *mut _ as *mut core::ffi::c_void,
        )
        .unwrap_or_else(|e| {
            eprintln!("Error querying file attributes for '{}':", file_path);
            eprintln!("Error Code: 0x{:08X}", e.code().0 as u32);
            eprintln!("Error Message: {}", e);
            std::process::exit(1);
        });
    }

    // Extract and display file attributes
    let attributes = file_info.dwFileAttributes;
    println!("File: {}", file_path);
    println!("Attributes: 0x{:08X}", attributes);

    // Display attribute flags
    if attributes & 0x00000001 != 0 {
        println!("  - FILE_ATTRIBUTE_READONLY");
    }
    if attributes & 0x00000002 != 0 {
        println!("  - FILE_ATTRIBUTE_HIDDEN");
    }
    if attributes & 0x00000004 != 0 {
        println!("  - FILE_ATTRIBUTE_SYSTEM");
    }
    if attributes & 0x00000010 != 0 {
        println!("  - FILE_ATTRIBUTE_DIRECTORY");
    }
    if attributes & 0x00000020 != 0 {
        println!("  - FILE_ATTRIBUTE_ARCHIVE");
    }
    if attributes & 0x00000800 != 0 {
        println!("  - FILE_ATTRIBUTE_REPARSE_POINT");
    }

    // Display timestamps
    let creation_time = file_info.ftCreationTime;
    let last_access_time = file_info.ftLastAccessTime;
    let last_write_time = file_info.ftLastWriteTime;

    println!("\nTimestamps:");
    println!("  Creation Time: {:?}", creation_time);
    println!("  Last Access Time: {:?}", last_access_time);
    println!("  Last Write Time: {:?}", last_write_time);

    // Display file size
    let file_size = file_info.nFileSizeLow as u64 | ((file_info.nFileSizeHigh as u64) << 32);
    println!("\nFile Size: {} bytes", file_size);
}
