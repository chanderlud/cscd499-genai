#![deny(warnings)]

use windows::core::Result;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Memory::AllocateUserPhysicalPages;

#[allow(dead_code)]
fn call_allocate_user_physical_pages() -> Result<Result<()>> {
    let mut number_of_pages: usize = 1;
    let mut page_array: usize = 0;
    let hprocess = HANDLE::default();

    // SAFETY: We pass valid mutable pointers to local variables that the API expects.
    unsafe {
        AllocateUserPhysicalPages(hprocess, &mut number_of_pages, &mut page_array)?;
    }
    Ok(Ok(()))
}
