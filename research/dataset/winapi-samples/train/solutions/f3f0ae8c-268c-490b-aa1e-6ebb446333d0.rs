use windows::core::{Result, HRESULT};
use windows::Win32::Foundation::{CloseHandle, ERROR_NO_MORE_ITEMS, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Heap32First, Heap32ListFirst, Heap32ListNext, Heap32Next,
    HEAPENTRY32, HEAPLIST32, HF32_DEFAULT, TH32CS_SNAPHEAPLIST,
};

#[derive(Debug, Clone)]
pub struct HeapRegion {
    pub base_address: usize,
    pub size: usize,
}

#[derive(Debug, Clone)]
pub struct HeapInfo {
    pub address: usize,
    pub regions: Vec<HeapRegion>,
}

pub fn enumerate_process_heaps(_handle: HANDLE, process_id: u32) -> Result<Vec<HeapInfo>> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPHEAPLIST, process_id) }?;

    let _snapshot_guard = HandleGuard(snapshot);

    let mut heaps = Vec::new();
    let mut heap_list = HEAPLIST32 {
        dwSize: std::mem::size_of::<HEAPLIST32>(),
        ..Default::default()
    };

    let mut heap_list_result = unsafe { Heap32ListFirst(snapshot, &mut heap_list) };

    while heap_list_result.is_ok() {
        if heap_list.dwFlags == HF32_DEFAULT {
            let mut regions = Vec::new();
            let mut heap_entry = HEAPENTRY32 {
                dwSize: std::mem::size_of::<HEAPENTRY32>(),
                ..Default::default()
            };

            let mut entry_result = unsafe {
                Heap32First(
                    &mut heap_entry,
                    heap_list.th32ProcessID,
                    heap_list.th32HeapID,
                )
            };

            while entry_result.is_ok() {
                regions.push(HeapRegion {
                    base_address: heap_entry.dwAddress,
                    size: heap_entry.dwBlockSize,
                });

                entry_result = unsafe { Heap32Next(&mut heap_entry) };
            }

            if let Err(e) = entry_result {
                if e.code() != HRESULT::from_win32(ERROR_NO_MORE_ITEMS.0) {
                    return Err(e);
                }
            }

            heaps.push(HeapInfo {
                address: heap_list.th32HeapID,
                regions,
            });
        }

        heap_list_result = unsafe { Heap32ListNext(snapshot, &mut heap_list) };
    }

    if let Err(e) = heap_list_result {
        if e.code() != HRESULT::from_win32(ERROR_NO_MORE_ITEMS.0) {
            return Err(e);
        }
    }

    Ok(heaps)
}

struct HandleGuard(HANDLE);

impl Drop for HandleGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}
