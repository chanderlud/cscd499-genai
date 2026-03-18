use std::os::windows::io::AsRawHandle;
use windows::core::{Error, Result};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::FlushFileBuffers;
use windows::Win32::System::Memory::FlushViewOfFile;

pub fn flush_mapped(view_ptr: *const u8, len: usize, file: &impl AsRawHandle) -> Result<()> {
    // Flush the memory-mapped view to disk
    // SAFETY: Caller guarantees view_ptr and len are valid for a mapped view
    unsafe { FlushViewOfFile(view_ptr as *const core::ffi::c_void, len) }?;

    // Flush the file buffers for stronger durability
    // SAFETY: Caller guarantees file handle is valid
    unsafe { FlushFileBuffers(HANDLE(file.as_raw_handle())) }?;

    Ok(())
}
