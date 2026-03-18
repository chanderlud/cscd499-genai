use windows::core::{Result, HRESULT};
use windows::Win32::Foundation::{CloseHandle, ERROR_NO_MORE_FILES, HANDLE, INVALID_HANDLE_VALUE};
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

pub fn enumerate_heap_blocks(process_id: u32) -> Result<Vec<HeapBlock>> {
    // Create snapshot handle - use ? to extract HANDLE from Result
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPHEAPLIST, process_id) }?;

    // Ensure handle is closed even if we return early
    struct SnapshotHandle(HANDLE);
    impl Drop for SnapshotHandle {
        fn drop(&mut self) {
            if self.0 != INVALID_HANDLE_VALUE {
                unsafe { CloseHandle(self.0) };
            }
        }
    }
    let _guard = SnapshotHandle(snapshot);

    let mut heap_blocks = Vec::new();

    // Initialize heap list structure
    let mut heap_list = HEAPLIST32::default();
    heap_list.dwSize = std::mem::size_of::<HEAPLIST32>();

    // Get first heap list - snapshot is now a HANDLE, not Result
    let mut heap_list_result = unsafe { Heap32ListFirst(snapshot, &mut heap_list) };

    while heap_list_result.is_ok() {
        // Initialize heap entry structure for this heap list
        let mut heap_entry = HEAPENTRY32::default();
        heap_entry.dwSize = std::mem::size_of::<HEAPENTRY32>();

        // Get first heap entry in this heap list
        let mut heap_entry_result = unsafe {
            Heap32First(
                &mut heap_entry,
                heap_list.th32ProcessID,
                heap_list.th32HeapID,
            )
        };

        while heap_entry_result.is_ok() {
            heap_blocks.push(HeapBlock {
                address: heap_entry.dwAddress as usize,
                size: heap_entry.dwBlockSize as usize,
                flags: heap_entry.dwFlags.0,
            });

            // Get next heap entry
            heap_entry_result = unsafe { Heap32Next(&mut heap_entry) };
        }

        // Check if we stopped because no more entries or an error
        if let Err(e) = heap_entry_result {
            let hr = e.code();
            if hr != HRESULT::from_win32(ERROR_NO_MORE_FILES.0) {
                return Err(e);
            }
        }

        // Get next heap list - snapshot is now a HANDLE, not Result
        heap_list_result = unsafe { Heap32ListNext(snapshot, &mut heap_list) };
    }

    // Check if we stopped because no more heap lists or an error
    if let Err(e) = heap_list_result {
        let hr = e.code();
        if hr != HRESULT::from_win32(ERROR_NO_MORE_FILES.0) {
            return Err(e);
        }
    }

    Ok(heap_blocks)
}
