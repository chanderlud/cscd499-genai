use windows::core::{Error, Result};
use windows::Win32::Foundation::{CloseHandle, ERROR_NO_MORE_FILES, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Heap32First, Heap32ListFirst, Heap32ListNext, Heap32Next,
    HEAPENTRY32, HEAPLIST32, TH32CS_SNAPHEAPLIST,
};

#[derive(Debug)]
pub struct HeapBlock {
    pub address: usize,
    pub size: usize,
    pub flags: u32,
}

pub fn for_each_heap_block<F>(process_id: u32, mut callback: F) -> Result<()>
where
    F: FnMut(HeapBlock) -> Result<()>,
{
    // Create toolhelp snapshot for heap lists
    // SAFETY: CreateToolhelp32Snapshot returns a handle that must be closed
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPHEAPLIST, process_id) }?;

    // Ensure handle is closed when function exits
    struct HandleGuard(HANDLE);
    impl Drop for HandleGuard {
        fn drop(&mut self) {
            // SAFETY: We own this handle and it was created by CreateToolhelp32Snapshot
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }
    let _guard = HandleGuard(snapshot);

    // Initialize heap list structure
    let mut heap_list = HEAPLIST32 {
        dwSize: std::mem::size_of::<HEAPLIST32>(),
        ..Default::default()
    };

    // Get first heap list
    // SAFETY: heap_list is properly initialized and we pass a valid snapshot handle
    let success = unsafe { Heap32ListFirst(snapshot, &mut heap_list) }.is_ok();
    if !success {
        let err = Error::from_thread();
        // ERROR_NO_MORE_FILES means no heap lists found (not an error for us)
        if err.code() == ERROR_NO_MORE_FILES.to_hresult() {
            return Ok(());
        }
        return Err(err);
    }

    // Iterate through all heap lists
    loop {
        // Initialize heap entry structure for this heap list
        let mut heap_entry = HEAPENTRY32 {
            dwSize: std::mem::size_of::<HEAPENTRY32>(),
            ..Default::default()
        };

        // Get first heap entry in this list
        // SAFETY: heap_entry is properly initialized and we pass valid parameters
        let success = unsafe {
            Heap32First(
                &mut heap_entry,
                heap_list.th32ProcessID,
                heap_list.th32HeapID,
            )
        }
        .is_ok();

        if success {
            // Iterate through all heap entries in this list
            loop {
                // Create HeapBlock from heap entry data
                let block = HeapBlock {
                    address: heap_entry.dwAddress,
                    size: heap_entry.dwBlockSize,
                    flags: heap_entry.dwFlags.0,
                };

                // Invoke callback with the block
                callback(block)?;

                // Get next heap entry
                // SAFETY: heap_entry is properly initialized
                let has_next = unsafe { Heap32Next(&mut heap_entry) }.is_ok();
                if !has_next {
                    let err = Error::from_thread();
                    // ERROR_NO_MORE_FILES means we've processed all entries
                    if err.code() != ERROR_NO_MORE_FILES.to_hresult() {
                        return Err(err);
                    }
                    break;
                }
            }
        } else {
            let err = Error::from_thread();
            // ERROR_NO_MORE_FILES means no entries in this heap list (not an error)
            if err.code() != ERROR_NO_MORE_FILES.to_hresult() {
                return Err(err);
            }
        }

        // Get next heap list
        // SAFETY: heap_list is properly initialized
        let has_next = unsafe { Heap32ListNext(snapshot, &mut heap_list) }.is_ok();
        if !has_next {
            let err = Error::from_thread();
            // ERROR_NO_MORE_FILES means we've processed all heap lists
            if err.code() != ERROR_NO_MORE_FILES.to_hresult() {
                return Err(err);
            }
            break;
        }
    }

    Ok(())
}
