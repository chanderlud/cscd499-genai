use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{HANDLE, CloseHandle, INVALID_HANDLE_VALUE};
use windows::Win32::System::Memory::{CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_ALL_ACCESS, PAGE_READWRITE};
use windows::Win32::System::Threading::{CreateEventW, SetEvent, WaitForSingleObject, INFINITE};
use std::sync::Arc;
use std::thread;

#[repr(C)]
struct RingBufferHeader {
    capacity: u32,
    read: u32,
    write: u32,
}

struct RingBuffer {
    header: *mut RingBufferHeader,
    data: *mut u8,
}

impl RingBuffer {
    unsafe fn new(base_ptr: *mut u8, capacity: u32) -> Self {
        let header = base_ptr as *mut RingBufferHeader;
        (*header).capacity = capacity;
        (*header).read = 0;
        (*header).write = 0;
        let data = base_ptr.add(std::mem::size_of::<RingBufferHeader>());
        RingBuffer { header, data }
    }

    unsafe fn available_read(&self) -> u32 {
        let read = (*self.header).read;
        let write = (*self.header).write;
        if write >= read {
            write - read
        } else {
            (*self.header).capacity - read + write
        }
    }

    unsafe fn available_write(&self) -> u32 {
        (*self.header).capacity - 1 - self.available_read()
    }

    unsafe fn push(&mut self, data: &[u8]) -> u32 {
        let mut written = 0;
        let capacity = (*self.header).capacity;
        let mut write = (*self.header).write;
        let read = (*self.header).read;

        while written < data.len() {
            let available = if write >= read {
                capacity - write + if read > 0 { read - 1 } else { 0 }
            } else {
                read - 1 - write
            };

            if available == 0 {
                break;
            }

            let to_write = std::cmp::min(available, (data.len() - written) as u32);
            let first_chunk = std::cmp::min(to_write, capacity - write);
            
            std::ptr::copy_nonoverlapping(
                data.as_ptr().add(written),
                self.data.add(write as usize),
                first_chunk as usize,
            );

            if first_chunk < to_write {
                std::ptr::copy_nonoverlapping(
                    data.as_ptr().add(written + first_chunk as usize),
                    self.data,
                    (to_write - first_chunk) as usize,
                );
            }

            write = (write + to_write) % capacity;
            written += to_write as usize;
        }

        (*self.header).write = write;
        written as u32
    }

    unsafe fn pop(&mut self, buffer: &mut [u8]) -> u32 {
        let mut read = 0;
        let capacity = (*self.header).capacity;
        let mut read_pos = (*self.header).read;
        let write = (*self.header).write;

        while read < buffer.len() {
            let available = if write >= read_pos {
                write - read_pos
            } else {
                capacity - read_pos + write
            };

            if available == 0 {
                break;
            }

            let to_read = std::cmp::min(available, (buffer.len() - read) as u32);
            let first_chunk = std::cmp::min(to_read, capacity - read_pos);
            
            std::ptr::copy_nonoverlapping(
                self.data.add(read_pos as usize),
                buffer.as_mut_ptr().add(read),
                first_chunk as usize,
            );

            if first_chunk < to_read {
                std::ptr::copy_nonoverlapping(
                    self.data,
                    buffer.as_mut_ptr().add(read + first_chunk as usize),
                    (to_read - first_chunk) as usize,
                );
            }

            read_pos = (read_pos + to_read) % capacity;
            read += to_read as usize;
        }

        (*self.header).read = read_pos;
        read as u32
    }
}

unsafe impl Send for RingBuffer {}
unsafe impl Sync for RingBuffer {}

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

pub fn shm_ringbuffer_roundtrip(base_name: &str, capacity: u32, chunks: &[Vec<u8>]) -> std::io::Result<Vec<u8>> {
    let total_size = std::mem::size_of::<RingBufferHeader>() + capacity as usize;
    let wide_name = wide_null(base_name);
    
    // Create shared memory
    let mapping = unsafe {
        CreateFileMappingW(
            HANDLE(INVALID_HANDLE_VALUE.0),
            None,
            PAGE_READWRITE,
            0,
            total_size as u32,
            PCWSTR(wide_name.as_ptr()),
        )?
    };
    
    let base_ptr = unsafe {
        MapViewOfFile(
            mapping,
            FILE_MAP_ALL_ACCESS,
            0,
            0,
            total_size,
        )
    };
    
    // Check if MapViewOfFile succeeded
    if base_ptr.Value.is_null() {
        return Err(windows::core::Error::from_win32().into());
    }
    
    // Initialize ring buffer
    let mut ring_buffer = unsafe { RingBuffer::new(base_ptr.Value as *mut u8, capacity) };
    
    // Create synchronization events
    let data_available = unsafe {
        CreateEventW(None, false, false, None)?
    };
    
    let space_available = unsafe {
        CreateEventW(None, false, true, None)?
    };
    
    let ring_buffer = Arc::new(ring_buffer);
    let data_available = Arc::new(data_available);
    let space_available = Arc::new(space_available);
    
    let producer_ring = Arc::clone(&ring_buffer);
    let producer_data = Arc::clone(&data_available);
    let producer_space = Arc::clone(&space_available);
    let chunks = chunks.to_vec();
    
    // Producer thread
    let producer = thread::spawn(move || -> Result<()> {
        for chunk in chunks {
            let mut offset = 0;
            while offset < chunk.len() {
                unsafe {
                    WaitForSingleObject(*producer_space, INFINITE);
                    
                    let written = producer_ring.push(&chunk[offset..]);
                    offset += written as usize;
                    
                    if written > 0 {
                        SetEvent(*producer_data)?;
                    }
                    
                    if offset < chunk.len() {
                        // Still have data to write, signal that we need more space
                        SetEvent(*producer_space)?;
                    }
                }
            }
        }
        Ok(())
    });
    
    let consumer_ring = Arc::clone(&ring_buffer);
    let consumer_data = Arc::clone(&data_available);
    let consumer_space = Arc::clone(&space_available);
    let total_bytes: usize = chunks.iter().map(|c| c.len()).sum();
    
    // Consumer thread
    let consumer = thread::spawn(move || -> Result<Vec<u8>> {
        let mut result = Vec::with_capacity(total_bytes);
        let mut buffer = vec![0u8; 1024];
        
        while result.len() < total_bytes {
            unsafe {
                WaitForSingleObject(*consumer_data, INFINITE);
                
                let read = consumer_ring.pop(&mut buffer);
                if read > 0 {
                    result.extend_from_slice(&buffer[..read as usize]);
                    SetEvent(*consumer_space)?;
                }
                
                if result.len() < total_bytes {
                    // Still need more data
                    SetEvent(*consumer_data)?;
                }
            }
        }
        
        Ok(result)
    });
    
    // Wait for threads to complete
    let producer_result = producer.join().map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::Other, "Producer thread panicked")
    })?;
    producer_result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    
    let consumer_result = consumer.join().map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::Other, "Consumer thread panicked")
    })?;
    let consumer_result = consumer_result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    
    // Clean up
    unsafe {
        CloseHandle(*data_available)?;
        CloseHandle(*space_available)?;
        UnmapViewOfFile(base_ptr)?;
        CloseHandle(mapping)?;
    }
    
    Ok(consumer_result)
}
