use windows::core::Result;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Memory::{VirtualProtectEx, PAGE_PROTECTION_FLAGS};

pub fn change_memory_protection(
    handle: HANDLE,
    address: u32,
    size: usize,
    new_protection: PAGE_PROTECTION_FLAGS,
) -> Result<PAGE_PROTECTION_FLAGS> {
    let mut old_protection = PAGE_PROTECTION_FLAGS(0);

    // SAFETY: We're calling VirtualProtectEx with valid parameters:
    // - handle is a valid process handle
    // - address is converted to a pointer (valid for target process)
    // - size is the region size
    // - new_protection is a valid protection constant
    // - old_protection is a valid output pointer
    unsafe {
        VirtualProtectEx(
            handle,
            address as *const core::ffi::c_void,
            size,
            new_protection,
            &mut old_protection,
        )?;
    }

    Ok(old_protection)
}
