use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::S_OK;
use windows::Win32::System::Com::{BindMoniker, IMoniker};

fn call_bind_moniker() -> HRESULT {
    let result: Result<IMoniker> = unsafe { BindMoniker(None::<&IMoniker>, 0) };
    result.map(|_| S_OK).unwrap_or_else(|e: Error| e.code())
}
