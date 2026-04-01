use windows::core::IUnknown;
use windows::core::GUID;
use windows::core::{Error, Result};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};

fn call_co_create_instance() -> Result<IUnknown> {
    // Use a well-known CLSID for demonstration
    let clsid = GUID::from_u128(0x00000000_0000_0000_C000_000000000046); // CLSID_StdMarshal

    unsafe { CoCreateInstance(&clsid, None, CLSCTX_INPROC_SERVER) }
}
