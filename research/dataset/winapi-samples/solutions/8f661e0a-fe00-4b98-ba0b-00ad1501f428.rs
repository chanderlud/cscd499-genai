use windows::core::{Error, Result};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{VIRTUAL_KEY, VK_CONTROL, VK_RMENU};
use windows::Win32::UI::WindowsAndMessaging::{
    PeekMessageW, MSG, PM_NOREMOVE, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

fn main() -> Result<()> {
    // Simulate receiving a keyboard message
    let hwnd = HWND::default();
    let wparam = WPARAM(VK_CONTROL.0 as usize);
    let lparam = LPARAM(0x001D0001); // Example lParam for Ctrl key press

    // Check if this Ctrl key event is fake (precedes AltGr)
    let is_fake = is_fake_ctrl_event(hwnd, wparam, lparam)?;

    println!("Is fake Ctrl event: {}", is_fake);
    Ok(())
}

/// Checks if the current Ctrl key event is fake (precedes an AltGr key event).
/// On Windows, AltGr is implemented as Ctrl+Alt, so the system generates
/// a fake Ctrl key press before every AltGr key press.
fn is_fake_ctrl_event(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> Result<bool> {
    // First, check if this is a Ctrl key event
    let vkey = VIRTUAL_KEY(wparam.0 as u16);
    if vkey != VK_CONTROL {
        return Ok(false);
    }

    // Check if this is a key down event (fake events only happen on key down)
    let is_key_down = is_key_down_message(lparam);
    if !is_key_down {
        return Ok(false);
    }

    // Peek at the next message without removing it from the queue
    let mut next_msg = std::mem::MaybeUninit::<MSG>::uninit();
    let has_next_message = unsafe {
        PeekMessageW(
            next_msg.as_mut_ptr(),
            Some(hwnd),
            WM_KEYDOWN,
            WM_SYSKEYUP,
            PM_NOREMOVE,
        )
    };

    if !has_next_message.as_bool() {
        return Ok(false);
    }

    // SAFETY: We checked that PeekMessageW returned true, so next_msg is initialized
    let next_msg = unsafe { next_msg.assume_init() };

    // Check if the next message is a right Alt (AltGr) key event
    let next_vkey = VIRTUAL_KEY(next_msg.wParam.0 as u16);
    let next_is_altgr = next_vkey == VK_RMENU;

    // If the next message is AltGr, then the current Ctrl event is fake
    Ok(next_is_altgr)
}

/// Determines if a keyboard message represents a key down event
fn is_key_down_message(lparam: LPARAM) -> bool {
    // In the lParam for keyboard messages:
    // Bit 31: 0 = key down, 1 = key up
    let transition_state = (lparam.0 >> 31) & 0x01;
    transition_state == 0
}
