use windows::core::{Error, Result};
use windows::Win32::System::WinRT::{CoDecodeProxy, ServerInformation};

fn call_co_decode_proxy() -> Result<Result<ServerInformation>> {
    Ok(unsafe { CoDecodeProxy(0, 0) })
}
