use std::path::Path;
use windows::core::{Result, Error, PCWSTR};
use windows::Win32::System::Com::{
    CoInitializeEx, COINIT_APARTMENTTHREADED, IStream,
    STGM_CREATE, STGM_READ, STGM_READWRITE, STGM_SHARE_EXCLUSIVE,
};
use windows::Win32::System::Com::StructuredStorage::{
    IStorage, StgCreateStorageEx, StgOpenStorageEx, STGFMT, STGFMT_STORAGE,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    s.encode_wide().chain(once(0)).collect()
}

pub fn stg_stream_roundtrip(path: &Path, stream_name: &str, data: &[u8]) -> Result<Vec<u8>> {
    // Initialize COM for this thread
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
    }

    let path_wide = wide_null(path.as_os_str());
    let stream_name_wide = wide_null(std::ffi::OsStr::new(stream_name));

    // Create compound file and write stream
    let storage: IStorage = unsafe {
        let mut storage = None;
        StgCreateStorageEx(
            PCWSTR(path_wide.as_ptr()),
            STGM_CREATE | STGM_READWRITE | STGM_SHARE_EXCLUSIVE,
            STGFMT_STORAGE,
            0,
            None,
            None,
            &IStorage::IID,
            &mut storage as *mut _ as *mut _,
        )?;
        storage.ok_or_else(|| Error::from_hresult(windows::Win32::Foundation::E_FAIL))?
    };

    let stream: IStream = unsafe {
        let mut stream = None;
        storage.CreateStream(
            PCWSTR(stream_name_wide.as_ptr()),
            STGM_READWRITE | STGM_SHARE_EXCLUSIVE,
            0,
            0,
            &mut stream,
        ).ok()?;
        stream.ok_or_else(|| Error::from_hresult(windows::Win32::Foundation::E_FAIL))?
    };

    // Write data to stream
    unsafe {
        let mut bytes_written = 0u32;
        stream.Write(
            data.as_ptr() as *const _,
            data.len() as u32,
            Some(&mut bytes_written),
        ).ok()?;
        if bytes_written as usize != data.len() {
            return Err(Error::from_hresult(windows::Win32::Foundation::E_FAIL));
        }
    }

    // Commit and release (drop handles)
    unsafe {
        storage.Commit(0).ok()?;
    }
    drop(stream);
    drop(storage);

    // Reopen compound file and read stream
    let storage: IStorage = unsafe {
        let mut storage = None;
        StgOpenStorageEx(
            PCWSTR(path_wide.as_ptr()),
            STGM_READ | STGM_SHARE_EXCLUSIVE,
            STGFMT_STORAGE,
            0,
            None,
            None,
            &IStorage::IID,
            &mut storage as *mut _ as *mut _,
        )?;
        storage.ok_or_else(|| Error::from_hresult(windows::Win32::Foundation::E_FAIL))?
    };

    let stream: IStream = unsafe {
        let mut stream = None;
        storage.OpenStream(
            PCWSTR(stream_name_wide.as_ptr()),
            None,
            STGM_READ | STGM_SHARE_EXCLUSIVE,
            0,
            &mut stream,
        ).ok()?;
        stream.ok_or_else(|| Error::from_hresult(windows::Win32::Foundation::E_FAIL))?
    };

    // Read data back
    let mut buffer = vec![0u8; data.len()];
    unsafe {
        let mut bytes_read = 0u32;
        stream.Read(
            buffer.as_mut_ptr() as *mut _,
            buffer.len() as u32,
            Some(&mut bytes_read),
        ).ok()?;
        if bytes_read as usize != data.len() {
            return Err(Error::from_hresult(windows::Win32::Foundation::E_FAIL));
        }
    }

    Ok(buffer)
}