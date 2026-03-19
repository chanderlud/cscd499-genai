use windows::core::{Error, Result, PCSTR};
use windows::Win32::Foundation::E_INVALIDARG;
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};
use windows::Win32::System::Memory::{
    VirtualAlloc, VirtualProtect, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE,
    PAGE_PROTECTION_FLAGS,
};

pub unsafe fn intercept_function(
    module_name: &str,
    function_name: &str,
    detour: *const std::ffi::c_void,
) -> Result<*const std::ffi::c_void> {
    // Convert to C strings for Win32 APIs
    let module_cstr =
        std::ffi::CString::new(module_name).map_err(|_| Error::from_hresult(E_INVALIDARG))?;

    let function_cstr =
        std::ffi::CString::new(function_name).map_err(|_| Error::from_hresult(E_INVALIDARG))?;

    // Load the target module - unsafe due to FFI call
    let module_handle = unsafe { LoadLibraryA(PCSTR(module_cstr.as_ptr() as *const u8)) }?;

    if module_handle.is_invalid() {
        return Err(Error::from_thread());
    }

    // Get the target function address - unsafe due to FFI call
    let target_func =
        unsafe { GetProcAddress(module_handle, PCSTR(function_cstr.as_ptr() as *const u8)) };

    let target_addr = match target_func {
        Some(func) => func as *mut u8,
        None => return Err(Error::from_thread()),
    };

    // Save original bytes (first 5 bytes of target function) - unsafe pointer operation
    let mut original_bytes = [0u8; 5];
    unsafe {
        std::ptr::copy_nonoverlapping(target_addr, original_bytes.as_mut_ptr(), 5);
    }

    // Change memory protection to allow writing - unsafe FFI call
    let mut old_protect = PAGE_PROTECTION_FLAGS(0);
    unsafe {
        VirtualProtect(
            target_addr as *const std::ffi::c_void,
            5,
            PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        )?;
    }

    // Calculate relative jump offset for detour
    let detour_addr = detour as usize;
    let target_addr_usize = target_addr as usize;
    let jump_offset = detour_addr.wrapping_sub(target_addr_usize).wrapping_sub(5);

    // Write jump instruction (0xE9) followed by relative offset - unsafe pointer operations
    unsafe {
        target_addr.write(0xE9);
        std::ptr::write_unaligned(target_addr.add(1) as *mut u32, jump_offset as u32);
    }

    // Restore original memory protection - unsafe FFI call
    let mut temp_protect = PAGE_PROTECTION_FLAGS(0);
    unsafe {
        let _ = VirtualProtect(
            target_addr as *const std::ffi::c_void,
            5,
            old_protect,
            &mut temp_protect,
        );
    }

    // Allocate executable memory for trampoline - unsafe FFI call
    let trampoline_size = 5 + 5; // Original bytes + jump instruction
    let trampoline = unsafe {
        VirtualAlloc(
            None,
            trampoline_size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        )
    };
    if trampoline.is_null() {
        return Err(Error::from_thread());
    }

    // Copy original bytes to trampoline - unsafe pointer operation
    unsafe {
        std::ptr::copy_nonoverlapping(original_bytes.as_ptr(), trampoline as *mut u8, 5);
    }

    // Calculate jump offset for trampoline to continue original function
    let continuation_addr = target_addr_usize + 5;
    let trampoline_jump_offset = continuation_addr
        .wrapping_sub(trampoline as usize + 5)
        .wrapping_sub(5);

    // Write jump instruction in trampoline - unsafe pointer operations
    unsafe {
        let jump_addr = (trampoline as *mut u8).add(5);
        jump_addr.write(0xE9);
        std::ptr::write_unaligned(jump_addr.add(1) as *mut u32, trampoline_jump_offset as u32);
    }

    Ok(trampoline as *const std::ffi::c_void)
}
