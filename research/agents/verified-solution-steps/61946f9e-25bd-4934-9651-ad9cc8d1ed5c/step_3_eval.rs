use windows::core::{Result, Error};
use windows::Win32::System::Com::{
    IStream, STREAM_SEEK_SET, CoInitializeEx, COINIT_MULTITHREADED,
    CoUninitialize,
};
use windows::Win32::System::Com::StructuredStorage::CreateStreamOnHGlobal;
use windows::Win32::Foundation::{E_UNEXPECTED, S_OK, S_FALSE, HGLOBAL};

pub fn hglobal_stream_roundtrip(data: &[u8]) -> Result<Vec<u8>> {
    // Initialize COM for this thread
    let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
    if hr != S_OK && hr != S_FALSE {
        return Err(Error::from_hresult(hr));
    }
    
    // Ensure COM is uninitialized when we're done
    struct ComUninitializer;
    impl Drop for ComUninitializer {
        fn drop(&mut self) {
            unsafe { CoUninitialize() };
        }
    }
    let _uninit = ComUninitializer;

    // Create stream on global memory
    let stream: IStream = unsafe { CreateStreamOnHGlobal(HGLOBAL::default(), true) }?;

    // Write data to stream
    let mut bytes_written = 0u32;
    unsafe {
        let hr = stream.Write(
            data.as_ptr() as *const _,
            data.len() as u32,
            Some(&mut bytes_written),
        );
        if hr.is_err() {
            return Err(Error::from_hresult(hr));
        }
    }

    // Seek to beginning of stream
    let mut new_position = 0u64;
    unsafe {
        let hr = stream.Seek(0, STREAM_SEEK_SET, Some(&mut new_position));
        if hr.is_err() {
            return Err(Error::from_hresult(hr));
        }
    }

    // Read data back from stream
    let mut buffer = vec![0u8; data.len()];
    let mut bytes_read = 0u32;
    unsafe {
        let hr = stream.Read(
            buffer.as_mut_ptr() as *mut _,
            data.len() as u32,
            Some(&mut bytes_read),
        );
        if hr.is_err() {
            return Err(Error::from_hresult(hr));
        }
    }

    // Verify we read the expected amount
    if bytes_read as usize != data.len() {
        return Err(Error::from_hresult(E_UNEXPECTED));
    }

    Ok(buffer)
}