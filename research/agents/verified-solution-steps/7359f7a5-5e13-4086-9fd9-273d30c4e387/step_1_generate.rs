use std::path::Path;
use windows::core::{Result, Error, PCWSTR, HRESULT};
use windows::Win32::System::Com::StructuredStorage::{
    StgCreateStorageEx, StgOpenStorageEx, IStorage, IStream,
    STGM_CREATE, STGM_READ, STGM_READWRITE, STGM_SHARE_EXCLUSIVE,
};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
use windows::Win32::Foundation::S_OK;

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn stg_stream_roundtrip(path: &Path, stream_name: &str, data: &[u8]) -> Result<Vec<u8>> {
    // Initialize COM for this thread
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)?;
    }

    let path_wide = wide_null(path.as_os_str());
    let stream_name_wide = wide_null(std::ffi::OsStr::new(stream_name));

    // Create compound file and write stream
    let storage = unsafe {
        let mut storage = None;
        let hr = StgCreateStorageEx(
            PCWSTR(path_wide.as_ptr()),
            STGM_CREATE | STGM_READWRITE | STGM_SHARE_EXCLUSIVE,
            0,
            0,
            None,
            None,
            &IStorage::IID,
            &mut storage as *mut _ as *mut _,
        );
        if hr != S_OK {
            return Err(Error::from_hresult(hr));
        }
        storage.ok_or_else(|| Error::from_hresult(HRESULT::from_win32(1)))? // E_FAIL
    };

    let stream = unsafe {
        let mut stream = None;
        let hr = storage.CreateStream(
            PCWSTR(stream_name_wide.as_ptr()),
            STGM_READWRITE | STGM_SHARE_EXCLUSIVE,
            0,
            0,
            &mut stream,
        );
        if hr != S_OK {
            return Err(Error::from_hresult(hr));
        }
        stream.ok_or_else(|| Error::from_hresult(HRESULT::from_win32(1)))?
    };

    // Write data to stream
    unsafe {
        let mut bytes_written = 0u32;
        let hr = stream.Write(
            data.as_ptr() as *const _,
            data.len() as u32,
            Some(&mut bytes_written),
        );
        if hr != S_OK {
            return Err(Error::from_hresult(hr));
        }
        if bytes_written as usize != data.len() {
            return Err(Error::from_hresult(HRESULT::from_win32(1)));
        }
    }

    // Commit and release (drop handles)
    unsafe {
        let hr = storage.Commit(0);
        if hr != S_OK {
            return Err(Error::from_hresult(hr));
        }
    }
    drop(stream);
    drop(storage);

    // Reopen compound file and read stream
    let storage = unsafe {
        let mut storage = None;
        let hr = StgOpenStorageEx(
            PCWSTR(path_wide.as_ptr()),
            STGM_READ | STGM_SHARE_EXCLUSIVE,
            0,
            0,
            None,
            None,
            &IStorage::IID,
            &mut storage as *mut _ as *mut _,
        );
        if hr != S_OK {
            return Err(Error::from_hresult(hr));
        }
        storage.ok_or_else(|| Error::from_hresult(HRESULT::from_win32(1)))?
    };

    let stream = unsafe {
        let mut stream = None;
        let hr = storage.OpenStream(
            PCWSTR(stream_name_wide.as_ptr()),
            None,
            STGM_READ | STGM_SHARE_EXCLUSIVE,
            0,
            &mut stream,
        );
        if hr != S_OK {
            return Err(Error::from_hresult(hr));
        }
        stream.ok_or_else(|| Error::from_hresult(HRESULT::from_win32(1)))?
    };

    // Read data back
    let mut buffer = vec![0u8; data.len()];
    unsafe {
        let mut bytes_read = 0u32;
        let hr = stream.Read(
            buffer.as_mut_ptr() as *mut _,
            buffer.len() as u32,
            Some(&mut bytes_read),
        );
        if hr != S_OK {
            return Err(Error::from_hresult(hr));
        }
        if bytes_read as usize != data.len() {
            return Err(Error::from_hresult(HRESULT::from_win32(1)));
        }
    }

    Ok(buffer)
}