use windows::core::{Error, Result};
use windows::Win32::Foundation::{E_UNEXPECTED, HGLOBAL, S_FALSE, S_OK};
use windows::Win32::System::Com::StructuredStorage::CreateStreamOnHGlobal;
use windows::Win32::System::Com::{
    CoInitializeEx, CoUninitialize, IStream, COINIT_MULTITHREADED, STREAM_SEEK_SET,
};

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

    // Write data to stream - convert HRESULT to Result using .ok()
    let mut bytes_written = 0u32;
    unsafe {
        stream
            .Write(
                data.as_ptr() as *const _,
                data.len() as u32,
                Some(&mut bytes_written),
            )
            .ok()?;
    }

    // Seek to beginning of stream - use ? operator
    let mut new_position = 0u64;
    unsafe {
        stream.Seek(0, STREAM_SEEK_SET, Some(&mut new_position))?;
    }

    // Read data back from stream - convert HRESULT to Result using .ok()
    let mut buffer = vec![0u8; data.len()];
    let mut bytes_read = 0u32;
    unsafe {
        stream
            .Read(
                buffer.as_mut_ptr() as *mut _,
                data.len() as u32,
                Some(&mut bytes_read),
            )
            .ok()?;
    }

    // Verify we read the expected amount
    if bytes_read as usize != data.len() {
        return Err(Error::from_hresult(E_UNEXPECTED));
    }

    Ok(buffer)
}