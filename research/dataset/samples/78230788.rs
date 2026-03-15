// TITLE: Read game menu active state from fixed memory address

use windows::core::{Error, Result};

/// Reads the GTA:SA menu active state from a fixed memory address.
///
/// # Safety
/// This function reads from a hardcoded memory address that must be valid
/// and contain a boolean value in the target process.
pub fn is_gta_menu_active() -> bool {
    // SAFETY: Reading from a fixed memory address in the GTA:SA process.
    // The address 0xBA67A4 is known to contain the menu active state.
    unsafe { *(0xBA67A4 as *const bool) }
}

fn main() -> Result<()> {
    let menu_active = is_gta_menu_active();
    println!("GTA menu active: {}", menu_active);

    Ok(())
}
