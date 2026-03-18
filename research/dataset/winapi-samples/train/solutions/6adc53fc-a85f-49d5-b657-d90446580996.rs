// Check for More Character Messages in the Queue

use std::mem::MaybeUninit;
use windows::core::Result;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    PeekMessageW, MSG, PM_NOREMOVE, WM_CHAR, WM_KEYFIRST, WM_KEYLAST, WM_SYSCHAR,
};

fn has_more_char_messages(hwnd: HWND) -> Result<bool> {
    let mut next_msg = MaybeUninit::<MSG>::uninit();

    // SAFETY: PeekMessageW is a valid Win32 API call. We pass a valid pointer to uninitialized memory.
    let has_message = unsafe {
        PeekMessageW(
            next_msg.as_mut_ptr(),
            Some(hwnd),
            WM_KEYFIRST,
            WM_KEYLAST,
            PM_NOREMOVE,
        )
    };

    if !has_message.as_bool() {
        return Ok(false);
    }

    // SAFETY: If PeekMessageW returned true, the MSG structure has been initialized.
    let next_msg = unsafe { next_msg.assume_init() };
    let next_msg_kind = next_msg.message;

    Ok(next_msg_kind == WM_CHAR || next_msg_kind == WM_SYSCHAR)
}

fn main() -> Result<()> {
    // Example usage with a null window handle (would typically be a real window handle)
    let hwnd = HWND::default();
    let more_coming = has_more_char_messages(hwnd)?;
    println!("More character messages coming: {}", more_coming);
    Ok(())
}
