use std::ptr::null_mut;
use windows::core::{Error, Result};
use windows::Win32::Foundation::EXCEPTION_ACCESS_VIOLATION;
use windows::Win32::System::Diagnostics::Debug::{
    AddVectoredExceptionHandler, RemoveVectoredExceptionHandler, EXCEPTION_CONTINUE_EXECUTION,
    EXCEPTION_CONTINUE_SEARCH, EXCEPTION_POINTERS,
};
use windows::Win32::System::Memory::{
    VirtualAlloc, VirtualFree, VirtualProtect, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_NOACCESS,
    PAGE_READWRITE,
};

thread_local! {
    static GUARDED_PAGE: std::cell::Cell<*mut u32> = std::cell::Cell::new(null_mut());
}

unsafe extern "system" fn veh_handler(exception_info: *mut EXCEPTION_POINTERS) -> i32 {
    // SAFETY: Called by Windows with valid exception_info pointer
    let exception_record = unsafe { (*exception_info).ExceptionRecord };
    if exception_record.is_null() {
        return EXCEPTION_CONTINUE_SEARCH;
    }

    // SAFETY: We just checked for null
    let record = unsafe { &*exception_record };
    if record.ExceptionCode == EXCEPTION_ACCESS_VIOLATION {
        // Check if this is a read/write violation on our guarded page
        if record.NumberParameters >= 2 {
            let access_type = record.ExceptionInformation[0];
            let faulting_address = record.ExceptionInformation[1] as *mut u32;

            // access_type: 0 = read, 1 = write, 8 = execute
            if access_type == 0 || access_type == 1 {
                let handled = GUARDED_PAGE.with(|page| {
                    let guarded_ptr = page.get();
                    if !guarded_ptr.is_null() && faulting_address == guarded_ptr {
                        // Change protection to PAGE_READWRITE
                        let mut old_protect = PAGE_READWRITE;
                        let result = unsafe {
                            VirtualProtect(
                                guarded_ptr as *const _,
                                std::mem::size_of::<u32>(),
                                PAGE_READWRITE,
                                &mut old_protect,
                            )
                        };

                        if result.is_ok() {
                            return true;
                        }
                    }
                    false
                });

                if handled {
                    return EXCEPTION_CONTINUE_EXECUTION;
                }
            }
        }
    }

    EXCEPTION_CONTINUE_SEARCH
}

pub fn veh_guarded_read_u32(value: u32) -> Result<u32> {
    // Allocate a single page (4096 bytes) with read-write access
    let page = unsafe { VirtualAlloc(None, 4096, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE) };

    if page.is_null() {
        return Err(Error::from_thread());
    }

    // Write the value to the start of the page
    let value_ptr = page as *mut u32;
    unsafe { *value_ptr = value };

    // Mark the page as NOACCESS
    let mut old_protect = PAGE_READWRITE;
    let result = unsafe { VirtualProtect(page as *const _, 4096, PAGE_NOACCESS, &mut old_protect) };

    if result.is_err() {
        let err = Error::from_thread();
        unsafe { VirtualFree(page, 0, MEM_RELEASE) };
        return Err(err);
    }

    // Store the page pointer in thread-local storage for the VEH
    GUARDED_PAGE.with(|cell| {
        cell.set(value_ptr);
    });

    // Install the vectored exception handler
    let veh_handle = unsafe { AddVectoredExceptionHandler(1, Some(veh_handler)) };

    if veh_handle.is_null() {
        let err = Error::from_thread();
        // Restore page protection before freeing
        let mut old_protect = PAGE_READWRITE;
        unsafe { VirtualProtect(page as *const _, 4096, PAGE_READWRITE, &mut old_protect) };
        unsafe { VirtualFree(page, 0, MEM_RELEASE) };
        GUARDED_PAGE.with(|cell| cell.set(null_mut()));
        return Err(err);
    }

    // Attempt to read the value - this will trigger the access violation
    // and our VEH will handle it by changing the page protection
    let read_value = unsafe { *value_ptr };

    // Clean up: remove VEH and free memory
    unsafe { RemoveVectoredExceptionHandler(veh_handle) };

    // Restore page protection before freeing (optional but clean)
    let mut old_protect = PAGE_READWRITE;
    unsafe { VirtualProtect(page as *const _, 4096, PAGE_READWRITE, &mut old_protect) };
    unsafe { VirtualFree(page, 0, MEM_RELEASE) };

    // Clear thread-local storage
    GUARDED_PAGE.with(|cell| cell.set(null_mut()));

    Ok(read_value)
}
