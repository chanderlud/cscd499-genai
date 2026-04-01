use windows::core::{Error, Result};
use windows::Win32::Foundation::{HANDLE, S_OK};
use windows::Win32::Security::SECURITY_ATTRIBUTES;
use windows::Win32::System::Pipes::CreatePipe;

fn call_create_pipe() -> windows::core::HRESULT {
    let mut hreadpipe: HANDLE = HANDLE(std::ptr::null_mut());
    let mut hwritepipe: HANDLE = HANDLE(std::ptr::null_mut());

    let mut security_attributes = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: std::ptr::null_mut(),
        bInheritHandle: true.into(),
    };

    unsafe {
        CreatePipe(
            &mut hreadpipe as *mut _,
            &mut hwritepipe as *mut _,
            Some(&security_attributes as *const _),
            0,
        )
        .map(|_| S_OK)
        .unwrap_or_else(|e: Error| e.code())
    }
}
