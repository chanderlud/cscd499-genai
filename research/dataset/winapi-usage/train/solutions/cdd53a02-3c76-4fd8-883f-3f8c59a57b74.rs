use windows::core::{Error, Result};
use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::System::Memory::AllocateUserPhysicalPages;

fn call_allocate_user_physical_pages() -> WIN32_ERROR {
    let mut num_pages: usize = 0;
    let mut page_array: usize = 0;
    // SAFETY: We pass valid mutable references for out parameters and a default handle.
    // The API call is wrapped in a match to safely convert the Result to WIN32_ERROR.
    match unsafe { AllocateUserPhysicalPages(HANDLE::default(), &mut num_pages, &mut page_array) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
