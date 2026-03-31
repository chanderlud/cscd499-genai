use windows::core::HRESULT;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Memory::AllocateUserPhysicalPages;

fn call_allocate_user_physical_pages() -> HRESULT {
    let mut number_of_pages: usize = 0;
    let mut page_array: usize = 0;
    // SAFETY: We pass valid mutable pointers to local variables.
    // The API call may fail due to an invalid handle, but we correctly convert the Result.
    unsafe {
        match AllocateUserPhysicalPages(HANDLE::default(), &mut number_of_pages, &mut page_array) {
            Ok(()) => HRESULT::default(),
            Err(e) => e.code(),
        }
    }
}
