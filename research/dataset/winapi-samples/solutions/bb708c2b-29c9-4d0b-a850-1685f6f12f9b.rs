use std::ffi::OsStr;
use std::iter::once;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::InvalidateRect;
use windows::Win32::UI::Controls::{EM_GETPASSWORDCHAR, EM_SETPASSWORDCHAR};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassExW, SendMessageW, TranslateMessage, MSG, WINDOW_EX_STYLE, WINDOW_STYLE,
    WM_DESTROY, WNDCLASSEXW, WS_BORDER, WS_CHILD, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        // Register window class
        let class_name = wide_null(OsStr::new("PasswordExampleClass"));
        let wc = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wndproc),
            hInstance: Default::default(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        if RegisterClassExW(&wc) == 0 {
            return Err(Error::from_thread());
        }

        // Create main window
        let window_title = wide_null(OsStr::new("Password Character Example"));
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_title.as_ptr()),
            WINDOW_STYLE(WS_VISIBLE.0 | WS_OVERLAPPEDWINDOW.0),
            100,
            100,
            400,
            200,
            None,
            None,
            None,
            None,
        )?;

        // Create edit control with password style
        let edit_class = wide_null(OsStr::new("EDIT"));
        let edit_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(edit_class.as_ptr()),
            PCWSTR(std::ptr::null()),
            WINDOW_STYLE(WS_VISIBLE.0 | WS_CHILD.0 | WS_BORDER.0 | 0x0020), // ES_PASSWORD = 0x0020
            50,
            50,
            200,
            25,
            Some(hwnd),
            None,
            None,
            None,
        )?;

        // Set password character to '*'
        let password_char = '*' as u16;
        let result = SendMessageW(
            edit_hwnd,
            EM_SETPASSWORDCHAR,
            Some(WPARAM(password_char as usize)),
            Some(LPARAM(0)),
        );
        if result == LRESULT(0) {
            // Invalidate to refresh the control
            InvalidateRect(Some(edit_hwnd), None, true);
        }

        // Retrieve and print the password character
        let retrieved_char = SendMessageW(
            edit_hwnd,
            EM_GETPASSWORDCHAR,
            Some(WPARAM(0)),
            Some(LPARAM(0)),
        );
        if retrieved_char != LRESULT(0) {
            let ch = char::from_u32(retrieved_char.0 as u32).unwrap_or('?');
            println!("Password character is: '{}'", ch);
        } else {
            println!("No password character set");
        }

        // Message loop
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Ok(())
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
