// Convert Scancode to Virtual Key and Determine Key Location

use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyboardLayout, MapVirtualKeyExW, HKL, MAPVK_VSC_TO_VK_EX, VIRTUAL_KEY, VK_ABNT_C2, VK_ADD,
    VK_CLEAR, VK_DECIMAL, VK_DELETE, VK_DIVIDE, VK_DOWN, VK_END, VK_HOME, VK_INSERT, VK_LCONTROL,
    VK_LEFT, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_MULTIPLY, VK_NEXT, VK_NUMPAD0, VK_NUMPAD1,
    VK_NUMPAD2, VK_NUMPAD3, VK_NUMPAD4, VK_NUMPAD5, VK_NUMPAD6, VK_NUMPAD7, VK_NUMPAD8, VK_NUMPAD9,
    VK_PRIOR, VK_RCONTROL, VK_RETURN, VK_RIGHT, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SUBTRACT, VK_UP,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyLocation {
    Left,
    Right,
    Numpad,
    Standard,
}

fn get_key_location_from_scancode(scancode: u16, hkl: HKL) -> KeyLocation {
    let extended = (scancode & 0xE000) == 0xE000;
    let vkey = unsafe {
        VIRTUAL_KEY(MapVirtualKeyExW(scancode as u32, MAPVK_VSC_TO_VK_EX, Some(hkl)) as u16)
    };

    match vkey {
        VK_LSHIFT | VK_LCONTROL | VK_LMENU | VK_LWIN => KeyLocation::Left,
        VK_RSHIFT | VK_RCONTROL | VK_RMENU | VK_RWIN => KeyLocation::Right,
        VK_RETURN if extended => KeyLocation::Numpad,
        VK_INSERT | VK_DELETE | VK_END | VK_DOWN | VK_NEXT | VK_LEFT | VK_CLEAR | VK_RIGHT
        | VK_HOME | VK_UP | VK_PRIOR => {
            if extended {
                KeyLocation::Standard
            } else {
                KeyLocation::Numpad
            }
        }
        VK_NUMPAD0 | VK_NUMPAD1 | VK_NUMPAD2 | VK_NUMPAD3 | VK_NUMPAD4 | VK_NUMPAD5
        | VK_NUMPAD6 | VK_NUMPAD7 | VK_NUMPAD8 | VK_NUMPAD9 | VK_DECIMAL | VK_DIVIDE
        | VK_MULTIPLY | VK_SUBTRACT | VK_ADD | VK_ABNT_C2 => KeyLocation::Numpad,
        _ => KeyLocation::Standard,
    }
}

fn main() -> windows::core::Result<()> {
    let hkl = unsafe { GetKeyboardLayout(0) };

    // Example scancodes to test
    let test_scancodes = [
        0x001E, // 'A' key (standard)
        0xE01C, // Enter key (extended, numpad)
        0x001C, // Enter key (standard)
        0xE04B, // Left arrow (extended, standard)
        0x004B, // Numpad 4 (standard, numpad)
        0x003A, // Caps Lock (standard)
    ];

    for &scancode in &test_scancodes {
        let location = get_key_location_from_scancode(scancode, hkl);
        println!("Scancode 0x{:04X} -> {:?}", scancode, location);
    }

    Ok(())
}
