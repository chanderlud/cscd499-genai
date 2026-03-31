use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE, WAIT_EVENT, WAIT_OBJECT_0,
    WAIT_TIMEOUT,
};
use windows::Win32::System::Threading::{CreateEventW, ResetEvent, SetEvent, WaitForSingleObject};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventRoundtrip {
    pub already_existed: bool,
    pub signaled_immediately: bool,
    pub reset_to_nonsignaled: bool,
}

struct HandleGuard(HANDLE);

impl Drop for HandleGuard {
    fn drop(&mut self) {
        // SAFETY: CloseHandle is a Win32 API that requires unsafe
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

const E_INVALIDARG_HRESULT: HRESULT = HRESULT(0x80070057u32 as i32);
const WAIT_FAILED_CODE: u32 = 0xFFFF_FFFF;

fn to_wide_null(s: &str) -> Result<Vec<u16>> {
    if s.encode_utf16().any(|unit| unit == 0) {
        return Err(Error::new(
            E_INVALIDARG_HRESULT,
            "event name contains an interior NUL",
        ));
    }

    Ok(s.encode_utf16().chain(std::iter::once(0)).collect())
}

fn wait_zero(handle: HANDLE) -> Result<WAIT_EVENT> {
    let status = unsafe { WaitForSingleObject(handle, 0) };
    if status.0 == WAIT_FAILED_CODE {
        return Err(Error::from_thread());
    }
    Ok(status)
}

pub fn roundtrip_named_manual_reset_event(name: &str) -> Result<EventRoundtrip> {
    let wide_name = to_wide_null(name)?;

    let handle = unsafe { CreateEventW(None, true, false, PCWSTR(wide_name.as_ptr()))? };
    let handle = HandleGuard(handle);

    let already_existed = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;

    unsafe {
        SetEvent(handle.0)?;
    }
    let after_set = wait_zero(handle.0)?;

    unsafe {
        ResetEvent(handle.0)?;
    }
    let after_reset = wait_zero(handle.0)?;

    Ok(EventRoundtrip {
        already_existed,
        signaled_immediately: after_set == WAIT_OBJECT_0,
        reset_to_nonsignaled: after_reset == WAIT_TIMEOUT,
    })
}
