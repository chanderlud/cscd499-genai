use windows::core::Result;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyboardState, VK_CAPITAL};

fn main() -> Result<()> {
    let kbd_state = get_kbd_state()?;
    let caps_lock_on = kbd_state[usize::from(VK_CAPITAL.0)] & 1 != 0;

    println!("Caps Lock is {}", if caps_lock_on { "ON" } else { "OFF" });
    Ok(())
}

fn get_kbd_state() -> Result<[u8; 256]> {
    let mut kbd_state = [0u8; 256];
    // SAFETY: GetKeyboardState writes to the provided buffer
    unsafe {
        GetKeyboardState(&mut kbd_state)?;
    }
    Ok(kbd_state)
}
