use std::ffi::c_void;
use windows::core::{IUnknown, Result};

/// Obtains a Direct3D9 device interface from a fixed memory address.
///
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer to a fixed memory address.
/// The caller must ensure that:
/// - The memory address `0xC97C28` contains a valid `IDirect3DDevice9` interface pointer.
/// - The pointer is valid in the current process context.
/// - The interface pointer is properly reference-counted and will remain valid for the duration of use.
pub unsafe fn get_d3d9_device() -> IUnknown {
    windows::core::Interface::from_raw(*(0xC97C28 as *const *mut c_void))
}

fn main() -> Result<()> {
    // SAFETY: This example assumes it's running in the context of GTA:SA
    // where the fixed memory address contains a valid Direct3D9 device.
    let _device = unsafe { get_d3d9_device() };

    // The device interface is now usable for Direct3D9 operations
    println!("Successfully obtained IDirect3DDevice9 interface (as IUnknown)");

    Ok(())
}
