use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{COLOR_WINDOW, HBRUSH};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, GetWindowTextLengthW,
    GetWindowTextW, PostQuitMessage, RegisterClassExW, SetWindowTextW, TranslateMessage,
    CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, ES_AUTOHSCROLL, MSG, WINDOW_EX_STYLE, WINDOW_STYLE,
    WM_CREATE, WM_DESTROY, WNDCLASSEXW, WS_BORDER, WS_CHILD, WS_OVERLAPPEDWINDOW, WS_TABSTOP,
    WS_VISIBLE,
};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let class_name = wide_null(OsStr::new("SampleWindowClass"));

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: Default::default(),
            hIcon: Default::default(),
            hCursor: Default::default(),
            hbrBackground: HBRUSH((COLOR_WINDOW.0 + 1) as *mut _),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            hIconSm: Default::default(),
        };

        if RegisterClassExW(&wc) == 0 {
            return Err(Error::from_thread());
        }

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(wide_null(OsStr::new("Text Example")).as_ptr()),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            None,
            None,
        )?;

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
        WM_CREATE => {
            // Create an edit control
            let edit_class = wide_null(OsStr::new("EDIT"));
            let edit_hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                PCWSTR(edit_class.as_ptr()),
                PCWSTR(wide_null(OsStr::new("Initial text")).as_ptr()),
                WS_CHILD
                    | WS_VISIBLE
                    | WS_BORDER
                    | WINDOW_STYLE(ES_AUTOHSCROLL as u32)
                    | WS_TABSTOP,
                50,
                50,
                200,
                25,
                Some(hwnd),
                None,
                None,
                None,
            )
            .expect("Failed to create edit control");

            // Set new text using SetWindowTextW
            let new_text = wide_null(OsStr::new("Hello from SetWindowTextW!"));
            SetWindowTextW(edit_hwnd, PCWSTR(new_text.as_ptr())).expect("Failed to set text");

            // Get text length using GetWindowTextLengthW
            let length = GetWindowTextLengthW(edit_hwnd);

            // Get text using GetWindowTextW
            let mut buffer = vec![0u16; (length + 1) as usize];
            let chars_written = GetWindowTextW(edit_hwnd, &mut buffer);

            if chars_written > 0 {
                // Convert to Rust string and print
                let text = String::from_utf16_lossy(&buffer[..chars_written as usize]);
                println!("Edit control text: {}", text);
                println!("Text length: {}", length);
            }

            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
