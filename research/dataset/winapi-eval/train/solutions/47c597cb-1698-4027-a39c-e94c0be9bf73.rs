use std::io;
use std::sync::{Arc, Mutex};
use std::thread;

use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::System::Memory::{
    CreateFileMappingW, FILE_MAP_ALL_ACCESS, MapViewOfFile, PAGE_READWRITE, UnmapViewOfFile,
};
use windows::Win32::System::Threading::{
    CreateEventW, INFINITE, ResetEvent, SetEvent, WaitForSingleObject,
};
use windows::core::{PCWSTR, Result};

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
    fn new(base_ptr: *mut u8, capacity: u32) -> Self {
        unsafe {
            let header = base_ptr as *mut RingBufferHeader;
            (*header).capacity = capacity;
            (*header).read = 0;
            (*header).write = 0;

            let data = base_ptr.add(std::mem::size_of::<RingBufferHeader>());
            Self { header, data }
        }
    }

    fn available_data(&self) -> u32 {
        unsafe {
            let capacity = (*self.header).capacity;
            let read = (*self.header).read;
            let write = (*self.header).write;

            if write >= read {
                write - read
            } else {
                capacity - read + write
            }
        }
    }

    fn available_space(&self) -> u32 {
        unsafe { (*self.header).capacity - self.available_data() - 1 }
    }

    fn is_empty(&self) -> bool {
        self.available_data() == 0
    }

    fn is_full(&self) -> bool {
        self.available_space() == 0
    }

    fn push(&mut self, data: &[u8]) -> u32 {
        unsafe {
            let mut written = 0usize;
            let capacity = (*self.header).capacity;
            let mut write = (*self.header).write;
            let read = (*self.header).read;

            while written < data.len() {
                // Keep one byte empty so read == write always means "empty", never "full".
                let available = if write >= read {
                    capacity - (write - read) - 1
                } else {
                    read - write - 1
                };

                if available == 0 {
                    break;
                }

                let to_write = available.min((data.len() - written) as u32);
                let first_chunk = to_write.min(capacity - write);

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
    }

    fn pop(&mut self, buffer: &mut [u8]) -> u32 {
        unsafe {
            let mut read_bytes = 0usize;
            let capacity = (*self.header).capacity;
            let mut read_pos = (*self.header).read;
            let write = (*self.header).write;

            while read_bytes < buffer.len() {
                let available = if write >= read_pos {
                    write - read_pos
                } else {
                    capacity - read_pos + write
                };

                if available == 0 {
                    break;
                }

                let to_read = available.min((buffer.len() - read_bytes) as u32);
                let first_chunk = to_read.min(capacity - read_pos);

                std::ptr::copy_nonoverlapping(
                    self.data.add(read_pos as usize),
                    buffer.as_mut_ptr().add(read_bytes),
                    first_chunk as usize,
                );

                if first_chunk < to_read {
                    std::ptr::copy_nonoverlapping(
                        self.data,
                        buffer.as_mut_ptr().add(read_bytes + first_chunk as usize),
                        (to_read - first_chunk) as usize,
                    );
                }

                read_pos = (read_pos + to_read) % capacity;
                read_bytes += to_read as usize;
            }

            (*self.header).read = read_pos;
            read_bytes as u32
        }
    }
}

unsafe impl Send for RingBuffer {}

#[derive(Clone, Copy)]
#[repr(transparent)]
struct SendHandle(HANDLE);

unsafe impl Send for SendHandle {}
unsafe impl Sync for SendHandle {}

impl SendHandle {
    fn wait(self) {
        unsafe {
            WaitForSingleObject(self.0, INFINITE);
        }
    }

    fn set(self) -> Result<()> {
        unsafe { SetEvent(self.0) }
    }

    fn reset(self) -> Result<()> {
        unsafe { ResetEvent(self.0) }
    }

    fn close(self) -> Result<()> {
        unsafe { CloseHandle(self.0) }
    }
}

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

fn sync_events(
    rb: &RingBuffer,
    data_available: SendHandle,
    space_available: SendHandle,
) -> Result<()> {
    if rb.is_empty() {
        data_available.reset()?;
    } else {
        data_available.set()?;
    }

    if rb.is_full() {
        space_available.reset()?;
    } else {
        space_available.set()?;
    }

    Ok(())
}

pub fn shm_ringbuffer_roundtrip(
    base_name: &str,
    capacity: u32,
    chunks: &[Vec<u8>],
) -> io::Result<Vec<u8>> {
    if capacity < 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "capacity must be at least 2",
        ));
    }

    let total_size = std::mem::size_of::<RingBufferHeader>() + capacity as usize;
    let wide_name = wide_null(base_name);

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

    let base_ptr = unsafe { MapViewOfFile(mapping, FILE_MAP_ALL_ACCESS, 0, 0, total_size) };

    if base_ptr.Value.is_null() {
        unsafe {
            let _ = CloseHandle(mapping);
        }
        return Err(windows::core::Error::from_thread().into());
    }

    let ring_buffer = Arc::new(Mutex::new(RingBuffer::new(
        base_ptr.Value as *mut u8,
        capacity,
    )));

    // manual-reset events: they represent current state, not one-shot notifications
    let data_available = SendHandle(unsafe { CreateEventW(None, true, false, None)? });
    let space_available = SendHandle(unsafe { CreateEventW(None, true, true, None)? });

    let producer_ring = Arc::clone(&ring_buffer);
    let producer_data = data_available;
    let producer_space = space_available;
    let producer_chunks = chunks.to_vec();

    let producer = thread::spawn(move || -> Result<()> {
        for chunk in producer_chunks {
            let mut offset = 0usize;

            while offset < chunk.len() {
                let written = {
                    let mut rb = producer_ring.lock().expect("producer mutex poisoned");
                    let written = rb.push(&chunk[offset..]);
                    sync_events(&rb, producer_data, producer_space)?;
                    written
                };

                if written == 0 {
                    producer_space.wait();
                    continue;
                }

                offset += written as usize;
            }
        }

        Ok(())
    });

    let consumer_ring = Arc::clone(&ring_buffer);
    let consumer_data = data_available;
    let consumer_space = space_available;
    let total_bytes: usize = chunks.iter().map(|c| c.len()).sum();

    let consumer = thread::spawn(move || -> Result<Vec<u8>> {
        let mut result = Vec::with_capacity(total_bytes);
        let mut buffer = vec![0u8; 1024];

        while result.len() < total_bytes {
            let read = {
                let mut rb = consumer_ring.lock().expect("consumer mutex poisoned");
                let read = rb.pop(&mut buffer);
                sync_events(&rb, consumer_data, consumer_space)?;
                read
            };

            if read == 0 {
                consumer_data.wait();
                continue;
            }

            result.extend_from_slice(&buffer[..read as usize]);
        }

        Ok(result)
    });

    let producer_result = producer
        .join()
        .map_err(|_| io::Error::other("producer thread panicked"))?;
    producer_result.map_err(|e| io::Error::other(e.to_string()))?;

    let consumer_result = consumer
        .join()
        .map_err(|_| io::Error::other("consumer thread panicked"))?;
    let consumer_result = consumer_result.map_err(|e| io::Error::other(e.to_string()))?;

    unsafe {
        data_available.close()?;
        space_available.close()?;
        UnmapViewOfFile(base_ptr)?;
        CloseHandle(mapping)?;
    }

    Ok(consumer_result)
}
