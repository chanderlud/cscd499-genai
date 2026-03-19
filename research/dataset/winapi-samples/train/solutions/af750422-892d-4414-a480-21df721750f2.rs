use windows::{
    core::{s, Result, PCSTR},
    Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA},
};

fn main() -> Result<()> {
    // Load uxtheme.dll dynamically
    let hmodule = unsafe { LoadLibraryA(s!("uxtheme.dll"))? };

    // Get function pointer by ordinal (135 = AllowDarkModeForApp)
    const UXTHEME_ALLOWDARKMODEFORAPP_ORDINAL: u16 = 135;
    let func_ptr = unsafe {
        GetProcAddress(
            hmodule,
            PCSTR(UXTHEME_ALLOWDARKMODEFORAPP_ORDINAL as usize as *const u8),
        )
    };

    match func_ptr {
        Some(_ptr) => {
            println!(
                "Successfully found function at ordinal {}",
                UXTHEME_ALLOWDARKMODEFORAPP_ORDINAL
            );
        }
        None => {
            println!(
                "Function not found at ordinal {}",
                UXTHEME_ALLOWDARKMODEFORAPP_ORDINAL
            );
        }
    }

    Ok(())
}
