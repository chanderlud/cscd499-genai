use std::ffi::c_void;
use std::io::Result;
use std::sync::atomic::{AtomicI32, Ordering};
use std::thread;

use windows::Win32::System::Threading::{
    FlsAlloc, FlsFree, FlsSetValue, FLS_OUT_OF_INDEXES,
};

// FLS destructor callback that increments a shared counter
unsafe extern "system" fn fls_destructor(value: *const c_void) {
    if !value.is_null() {
        let counter = value as *const AtomicI32;
        (*counter).fetch_add(1, Ordering::SeqCst);
    }
}

pub fn fls_destructor_count(threads: usize) -> Result<i32> {
    // Allocate shared counter on heap as AtomicI32
    let counter = Box::into_raw(Box::new(AtomicI32::new(0)));
    
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
        // Pass counter pointer as usize to avoid Send issues with raw pointers
        let counter_usize = counter as usize;
        let handle = thread::spawn(move || -> Result<()> {
            // Reconstruct pointer from usize inside thread
            let counter_ptr = counter_usize as *const c_void;
            // Set non-null value (counter pointer) into FLS slot
            unsafe { FlsSetValue(slot, Some(counter_ptr)) }?;
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
    let count = unsafe { (*counter).load(Ordering::SeqCst) };
    unsafe { drop(Box::from_raw(counter)); }
    
    Ok(count)
}