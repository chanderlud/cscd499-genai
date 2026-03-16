use std::ffi::c_void;
use std::io::Result;
use std::thread;

use windows::Win32::System::Threading::{
    FlsAlloc, FlsFree, FlsSetValue, FLS_OUT_OF_INDEXES,
};

// FLS destructor callback that increments a shared counter
unsafe extern "system" fn fls_destructor(value: *const c_void) {
    if !value.is_null() {
        let counter = value as *mut i32;
        windows::Win32::System::Threading::InterlockedIncrement(counter);
    }
}

pub fn fls_destructor_count(threads: usize) -> Result<i32> {
    // Allocate shared counter on heap
    let counter = Box::into_raw(Box::new(0i32));
    
    // Allocate FLS slot with destructor callback
    let slot = unsafe { FlsAlloc(Some(fls_destructor)) };
    if slot == FLS_OUT_OF_INDEXES {
        // Clean up counter before returning error
        unsafe { drop(Box::from_raw(counter)); }
        return Err(windows::core::Error::from_thread().into());
    }
    
    // Spawn threads that set values in the FLS slot
    let mut handles = Vec::with_capacity(threads);
    for _ in 0..threads {
        let counter_ptr = counter as *mut c_void;
        let handle = thread::spawn(move || -> Result<()> {
            // Set non-null value (counter pointer) into FLS slot
            // Wrap in Some() and cast to *const c_void
            unsafe { FlsSetValue(slot, Some(counter_ptr as *const c_void)) }?;
            // Thread exits here, triggering destructor callback
            Ok(())
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().map_err(|_| {
            windows::core::Error::from_thread()
        })??;
    }
    
    // Free the FLS slot
    unsafe { FlsFree(slot) }?;
    
    // Read final count and clean up
    let count = unsafe { *counter };
    unsafe { drop(Box::from_raw(counter)); }
    
    Ok(count)
}