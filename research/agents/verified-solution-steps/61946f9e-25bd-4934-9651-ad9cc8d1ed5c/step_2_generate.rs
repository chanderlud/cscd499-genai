use windows::core::{Result, Error, HRESULT};
use windows::Win32::System::Com::{
    IStream, STREAM_SEEK_SET, CoInitializeEx, COINIT_MULTITHREADED,
    CoUninitialize,
};
use windows::Win32::System::Com::StructuredStorage::CreateStreamOnHGlobal;
use windows::Win32::Foundation::{E_UNEXPECTED, S_OK, S_FALSE};

pub fn hglobal_stream_roundtrip(data: &[u8]) -> Result<Vec<u8>> {
    // Initialize COM for this thread
    // SAFETY: CoInitializeEx is safe to call, we check the return value
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
    // SAFETY: CreateStreamOnHGlobal is safe with null handle and true for delete on release
    let stream: IStream = unsafe { CreateStreamOnHGlobal(None, true) }?;

    // Write data to stream
    let mut bytes_written = 0u32;
    // SAFETY: We're passing a valid pointer to our data slice and a valid pointer for bytes_written
    unsafe {
        stream.Write(
            data.as_ptr() as *const _,
            data.len() as u32,
            Some(&mut bytes_written),
        )?;
    }

    // Seek to beginning of stream
    let mut new_position = 0u64;
    // SAFETY: We're passing a valid pointer for new_position
    unsafe {
        stream.Seek(0, STREAM_SEEK_SET, Some(&mut new_position))?;
    }

    // Read data back from stream
    let mut buffer = vec![0u8; data.len()];
    let mut bytes_read = 0u32;
    // SAFETY: We're passing a valid pointer to our buffer and a valid pointer for bytes_read
    unsafe {
        stream.Read(
            buffer.as_mut_ptr() as *mut _,
            data.len() as u32,
            Some(&mut bytes_read),
        )?;
    }

    // Verify we read the expected amount
    if bytes_read as usize != data.len() {
        return Err(Error::from_hresult(E_UNEXPECTED));
    }

    Ok(buffer)
}