use windows::core::Result;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Com::{BindMoniker, IMoniker};

fn call_bind_moniker() -> WIN32_ERROR {
    let result: Result<IMoniker> = unsafe { BindMoniker(None::<&IMoniker>, 0) };
    match result {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
