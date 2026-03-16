use std::os::windows::io::{AsRawHandle, OwnedHandle};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::FlushFileBuffers;
use windows::Win32::System::Memory::FlushViewOfFile;
use windows::core::Result;

pub fn flush_mapped(view_ptr: *const u8, len: usize, file: &OwnedHandle) -> Result<()> {
    // SAFETY: Caller guarantees view_ptr and len are valid for a mapped view
    unsafe {
        // Cast to *const c_void and use ? to propagate errors
        FlushViewOfFile(view_ptr as *const core::ffi::c_void, len)?;
    }

    // Convert OwnedHandle to HANDLE for Win32 API
    let raw_handle = file.as_raw_handle();
    let handle = HANDLE(raw_handle as _);

    // SAFETY: Caller guarantees file handle is valid
    unsafe {
        // Use ? to propagate errors
        FlushFileBuffers(handle)?;
    }

    Ok(())
}
