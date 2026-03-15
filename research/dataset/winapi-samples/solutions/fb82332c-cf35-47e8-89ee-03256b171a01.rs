use windows::core::{Error, Result};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyboardLayout, MapVirtualKeyExW, MAPVK_VSC_TO_VK_EX,
};
use windows::Win32::UI::WindowsAndMessaging::{
    PeekMessageW, MSG, PM_NOREMOVE, WM_KEYFIRST, WM_KEYLAST,
};

fn main() -> Result<()> {
    // Simulate checking for the next key message in the queue without removing it
    let mut next_msg = MSG::default();
    let has_next_key_message = unsafe {
        // PeekMessageW is unsafe because it reads from the message queue
        PeekMessageW(
            &mut next_msg,
            Some(HWND::default()),
            WM_KEYFIRST,
            WM_KEYLAST,
            PM_NOREMOVE,
        )
    };

    if has_next_key_message.as_bool() {
        // Extract scancode from lParam (bits 16-23)
        let scancode = ((next_msg.lParam.0 >> 16) & 0xFF) as u8;
        let extended = ((next_msg.lParam.0 >> 24) & 0x01) != 0;

        // Combine scancode with extended flag
        let ex_scancode = (scancode as u16) | (if extended { 0xE000 } else { 0 });

        // Get current keyboard layout
        let hkl = unsafe {
            // GetKeyboardLayout is unsafe because it accesses system state
            GetKeyboardLayout(0)
        };

        // Map scancode to virtual key code
        let vkey = unsafe {
            // MapVirtualKeyExW is unsafe because it performs keyboard layout mapping
            MapVirtualKeyExW(ex_scancode as u32, MAPVK_VSC_TO_VK_EX, Some(hkl))
        };

        println!(
            "Next key message: scancode=0x{:04X}, vkey=0x{:04X}",
            ex_scancode, vkey
        );
    } else {
        println!("No key messages in queue");
    }

    Ok(())
}
