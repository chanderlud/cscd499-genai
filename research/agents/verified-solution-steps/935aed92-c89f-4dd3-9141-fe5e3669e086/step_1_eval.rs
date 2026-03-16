use std::path::Path;
use std::io;
use std::slice;

use windows::core::{Result, Error, HRESULT, PCWSTR};
use windows::Win32::Foundation::{HANDLE, CloseHandle, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{CreateFileW, GetFileSizeEx, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, OPEN_EXISTING, FILE_GENERIC_READ};
use windows::Win32::System::Memory::{CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_READ, PAGE_READONLY};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

struct MappedFile {
    file_handle: HANDLE,
    mapping_handle: HANDLE,
    view: *mut std::ffi::c_void,
    file_size: usize,
}

impl MappedFile {
    fn new(path: &Path) -> io::Result<Self> {
        let wide_path = wide_null(path.as_os_str());
        
        // SAFETY: CreateFileW is called with valid parameters
        let file_handle = unsafe {
            CreateFileW(
                PCWSTR(wide_path.as_ptr()),
                FILE_GENERIC_READ.0,
                FILE_SHARE_READ,
                std::ptr::null(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                HANDLE::default(),
            )
        };
        
        if file_handle == INVALID_HANDLE_VALUE {
            return Err(Error::from_thread().into());
        }
        
        // Get file size
        let mut file_size_i64 = 0i64;
        // SAFETY: GetFileSizeEx is called with valid handle and pointer
        let success = unsafe { GetFileSizeEx(file_handle, &mut file_size_i64) };
        if !success.as_bool() {
            unsafe { CloseHandle(file_handle) };
            return Err(Error::from_thread().into());
        }
        
        let file_size = file_size_i64 as usize;
        
        // SAFETY: CreateFileMappingW is called with valid parameters
        let mapping_handle = unsafe {
            CreateFileMappingW(
                file_handle,
                std::ptr::null(),
                PAGE_READONLY,
                0,
                0,
                PCWSTR::null(),
            )
        };
        
        if mapping_handle.is_invalid() {
            unsafe { CloseHandle(file_handle) };
            return Err(Error::from_thread().into());
        }
        
        // SAFETY: MapViewOfFile is called with valid parameters
        let view = unsafe {
            MapViewOfFile(
                mapping_handle,
                FILE_MAP_READ,
                0,
                0,
                0,
            )
        };
        
        if view.is_null() {
            unsafe {
                CloseHandle(mapping_handle);
                CloseHandle(file_handle);
            }
            return Err(Error::from_thread().into());
        }
        
        Ok(Self {
            file_handle,
            mapping_handle,
            view,
            file_size,
        })
    }
    
    fn as_bytes(&self) -> &[u8] {
        // SAFETY: view is valid for file_size bytes and we have exclusive access
        unsafe { slice::from_raw_parts(self.view as *const u8, self.file_size) }
    }
}

impl Drop for MappedFile {
    fn drop(&mut self) {
        unsafe {
            if !self.view.is_null() {
                UnmapViewOfFile(self.view);
            }
            if self.mapping_handle != HANDLE::default() {
                CloseHandle(self.mapping_handle);
            }
            if self.file_handle != INVALID_HANDLE_VALUE {
                CloseHandle(self.file_handle);
            }
        }
    }
}

pub fn mmap_count_nonoverlapping(path: &Path, needle: &[u8]) -> io::Result<usize> {
    if needle.is_empty() {
        return Ok(0);
    }
    
    let mapped = MappedFile::new(path)?;
    let data = mapped.as_bytes();
    
    let mut count = 0;
    let mut pos = 0;
    
    while pos + needle.len() <= data.len() {
        if &data[pos..pos + needle.len()] == needle {
            count += 1;
            pos += needle.len();
        } else {
            pos += 1;
        }
    }
    
    Ok(count)
}