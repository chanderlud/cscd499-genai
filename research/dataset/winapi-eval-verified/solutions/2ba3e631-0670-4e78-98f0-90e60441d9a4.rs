use std::mem::ManuallyDrop;
use std::thread;
use windows::Win32::Foundation::{HGLOBAL, S_FALSE, S_OK};
use windows::Win32::System::Com::Marshal::CoMarshalInterThreadInterfaceInStream;
use windows::Win32::System::Com::StructuredStorage::{
    CoGetInterfaceAndReleaseStream, CreateStreamOnHGlobal,
};
use windows::Win32::System::Com::{
    COINIT, COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize, IStream, STREAM_SEEK_SET,
};
use windows::core::{Error, HRESULT, Interface, Result};

struct ComInit;

impl ComInit {
    fn new(dw: COINIT) -> Result<Self> {
        let hr = unsafe { CoInitializeEx(None, dw) };
        match hr {
            S_OK | S_FALSE => Ok(Self),
            _ => Err(Error::from(hr)),
        }
    }
}

impl Drop for ComInit {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

pub fn com_marshal_stream_roundtrip(data: &[u8]) -> Result<Vec<u8>> {
    // Use MTA for test/helper code so there is no STA message-pump deadlock.
    let _main_com = ComInit::new(COINIT_MULTITHREADED)?;

    let stream: IStream = unsafe { CreateStreamOnHGlobal(HGLOBAL::default(), true) }?;

    let marshaled_stream =
        unsafe { CoMarshalInterThreadInterfaceInStream(&IStream::IID, &stream)? };

    let marshaled_stream_raw = marshaled_stream.into_raw() as usize;
    let data_vec = data.to_vec();

    let worker = thread::spawn(move || -> Result<()> {
        let _worker_com = ComInit::new(COINIT_MULTITHREADED)?;

        let marshaled_stream =
            ManuallyDrop::new(unsafe { IStream::from_raw(marshaled_stream_raw as *mut _) });

        let worker_stream: IStream = unsafe { CoGetInterfaceAndReleaseStream(&*marshaled_stream)? };

        let mut bytes_written = 0u32;
        unsafe {
            worker_stream
                .Write(
                    data_vec.as_ptr() as *const _,
                    data_vec.len() as u32,
                    Some(&mut bytes_written),
                )
                .ok()?;
        }

        Ok(())
    });

    worker
        .join()
        .map_err(|_| Error::from_hresult(HRESULT::from_win32(1u32)))??;

    unsafe {
        stream.Seek(0, STREAM_SEEK_SET, None)?;
    }

    let mut buffer = vec![0u8; data.len()];
    let mut bytes_read = 0u32;
    unsafe {
        stream
            .Read(
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u32,
                Some(&mut bytes_read),
            )
            .ok()?;
    }

    buffer.truncate(bytes_read as usize);
    Ok(buffer)
}
