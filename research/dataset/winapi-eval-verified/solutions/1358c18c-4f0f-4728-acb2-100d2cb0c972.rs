#![allow(non_snake_case)]
use std::io;
use std::sync::Arc;
use std::thread;
use std::cell::UnsafeCell;
use windows::core::Result;
use windows::Win32::System::Threading::{
    InitializeSynchronizationBarrier, EnterSynchronizationBarrier,
    DeleteSynchronizationBarrier, InterlockedAdd64,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

struct SyncUnsafeCell<T>(UnsafeCell<T>);
unsafe impl<T> Sync for SyncUnsafeCell<T> {}
unsafe impl<T> Send for SyncUnsafeCell<T> {}

impl<T> SyncUnsafeCell<T> {
    fn new(value: T) -> Self {
        SyncUnsafeCell(UnsafeCell::new(value))
    }
    
    fn get(&self) -> *mut T {
        self.0.get()
    }
}

pub fn barrier_phased_counter(threads: usize, phases: i64) -> io::Result<i64> {
    if threads == 0 || phases <= 0 {
        return Ok(0);
    }

    // Initialize synchronization barrier
    let mut barrier = Default::default();
    unsafe {
        InitializeSynchronizationBarrier(&mut barrier, threads as i32, 0)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.message()))?;
    }

    // Shared counter using Arc for thread-safe reference counting
    let counter = Arc::new(SyncUnsafeCell::new(0i64));
    let mut handles = Vec::with_capacity(threads);

    // Spawn worker threads
    for _ in 0..threads {
        let counter_clone = Arc::clone(&counter);
        let barrier_ptr = &barrier as *const _ as *mut _;

        let handle = thread::spawn(move || -> Result<()> {
            for _ in 0..phases {
                // Increment counter using Win32 interlocked operation
                unsafe {
                    InterlockedAdd64(counter_clone.get(), 1);
                }

                // Wait at barrier
                unsafe {
                    EnterSynchronizationBarrier(barrier_ptr, 0);
                }
            }
            Ok(())
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().map_err(|_| {
            io::Error::new(io::ErrorKind::Other, "Thread panicked")
        })??;
    }

    // Clean up barrier
    unsafe {
        let result = DeleteSynchronizationBarrier(&mut barrier);
        if !result.as_bool() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to delete synchronization barrier",
            ));
        }
    }

    // Return final counter value
    Ok(unsafe { *counter.get() })
}
