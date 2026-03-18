use windows::core::{Error, BOOL};
use windows::Win32::Foundation::{FALSE, HWND, LPARAM, TRUE};
use windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled;
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowTextLengthW, GetWindowTextW};

fn main() -> windows::core::Result<()> {
    // Callback for EnumWindows - must be 'static + Send + Sync for use in EnumWindows
    unsafe extern "system" fn enum_callback(hwnd: HWND, _lparam: LPARAM) -> BOOL {
        if IsWindowEnabled(hwnd) == FALSE {
            return TRUE;
        }

        let length = GetWindowTextLengthW(hwnd);
        if length == 0 {
            return TRUE;
        }

        // Allocate buffer with exact size and null terminator
        let mut buffer = vec![0u16; (length as usize) + 1];
        let result = GetWindowTextW(hwnd, &mut buffer[0..(length + 1) as usize]);
        if result < length + 1 {
            return TRUE;
        }

        // Convert to Rust string safely
        let title = String::from_utf16_lossy(&buffer[..length as usize]);
        println!("Enabled window: {}", title);

        TRUE
    }

    // Enumerate windows
    let result = unsafe { EnumWindows(Some(enum_callback), LPARAM(0)) };
    if result.is_err() {
        return Err(Error::from_thread());
    }

    Ok(())
}
