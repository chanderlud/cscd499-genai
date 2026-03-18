// Convert Virtual Key to Scan Code Using MapVirtualKeyExW

use windows::core::Result;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyboardLayout, MapVirtualKeyExW, HKL, MAPVK_VK_TO_VSC_EX, VIRTUAL_KEY, VK_A,
};

fn virtual_key_to_scan_code(vk: VIRTUAL_KEY, hkl: HKL) -> Result<u16> {
    // SAFETY: MapVirtualKeyExW is a valid Win32 API call
    let scan_code = unsafe { MapVirtualKeyExW(vk.0 as u32, MAPVK_VK_TO_VSC_EX, Some(hkl)) };

    if scan_code == 0 {
        // If the function fails, the return value is zero
        // MapVirtualKeyExW doesn't set last error, so use a generic error
        Err(windows::core::Error::new(
            windows::Win32::Foundation::E_FAIL,
            "MapVirtualKeyExW failed to convert virtual key to scan code",
        ))
    } else {
        Ok(scan_code as u16)
    }
}

fn main() -> Result<()> {
    // Get the current keyboard layout for the thread
    let hkl = unsafe { GetKeyboardLayout(0) };

    // Convert virtual key VK_A to its scan code
    let vk = VK_A;
    let scan_code = virtual_key_to_scan_code(vk, hkl)?;

    println!(
        "Virtual key {:?} maps to scan code: 0x{:04X}",
        vk, scan_code
    );

    Ok(())
}
