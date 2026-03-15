use std::ffi::c_void;
use windows::core::{IUnknown, Result};

pub unsafe fn get_d3d9_device() -> IUnknown {
    // SAFETY: Caller must ensure the memory address 0xC97C28 contains a valid
    // IDirect3DDevice9 interface pointer in the target process context.
    // We cast to IUnknown since IDirect3DDevice9 inherits from IUnknown.
    windows::core::Interface::from_raw(*(0xC97C28 as *const *mut c_void))
}

fn main() -> Result<()> {
    // SAFETY: This example assumes it's running in the context of GTA:SA
    // where the fixed memory address contains a valid Direct3D9 device.
    let device = unsafe { get_d3d9_device() };

    // The device interface is now usable for Direct3D9 operations
    println!("Successfully obtained IDirect3DDevice9 interface (as IUnknown)");

    Ok(())
}
