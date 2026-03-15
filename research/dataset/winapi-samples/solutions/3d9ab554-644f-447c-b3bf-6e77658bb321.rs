// TITLE: Read game window handle from fixed memory address

use windows::Win32::Foundation::HWND;

fn get_window_handle() -> HWND {
    // SAFETY: Reading from a fixed memory address that is known to contain
    // the game's window handle pointer in GTA:SA
    unsafe { **(0xC17054 as *const *const HWND) }
}

fn main() {
    let hwnd = get_window_handle();
    println!("Window handle: {:?}", hwnd);
}
