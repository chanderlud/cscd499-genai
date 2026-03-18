use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyState, VIRTUAL_KEY, VK_CAPITAL, VK_LSHIFT, VK_NUMLOCK, VK_SCROLL,
};

fn get_key_state(vk: VIRTUAL_KEY) -> u8 {
    let mut state = 0;
    unsafe {
        let async_state = GetAsyncKeyState(i32::from(vk.0));
        if (async_state & (1 << 15)) != 0 {
            state |= 0x80;
        }

        if matches!(vk, VK_CAPITAL | VK_NUMLOCK | VK_SCROLL) {
            let toggle_state = GetKeyState(i32::from(vk.0));
            if (toggle_state & 1) != 0 {
                state |= 0x01;
            }
        }
    }
    state
}

fn main() {
    let lshift_state = get_key_state(VK_LSHIFT);
    let caps_state = get_key_state(VK_CAPITAL);

    println!("Left Shift is down: {}", (lshift_state & 0x80) != 0);
    println!("Caps Lock is on: {}", (caps_state & 0x01) != 0);
    println!("Caps Lock is down: {}", (caps_state & 0x80) != 0);
}
