use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassW, ShowWindow, TranslateMessage, BS_NOTIFY, MSG, SW_SHOW, WINDOW_EX_STYLE,
    WINDOW_STYLE, WM_DESTROY, WM_LBUTTONDOWN, WNDCLASSW, WS_CHILD, WS_TABSTOP, WS_VISIBLE,
};

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            WM_LBUTTONDOWN => {
                println!("Button clicked!");
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

fn main() -> Result<()> {
    unsafe {
        // Register window class
        let class_name = wide_null("SampleButtonClass");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: Default::default(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        if RegisterClassW(&wc) == 0 {
            return Err(Error::from_thread());
        }

        // Create main window
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(wide_null("Button Example").as_ptr()),
            WINDOW_STYLE(WS_VISIBLE.0),
            100,
            100,
            400,
            300,
            None,
            None,
            None,
            None,
        )?;

        // Create button with styles
        let button_styles =
            WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | WS_TABSTOP.0 | (BS_NOTIFY as u32));
        let button_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(wide_null("BUTTON").as_ptr()),
            PCWSTR(wide_null("Click Me").as_ptr()),
            button_styles,
            50,
            50,
            120,
            30,
            Some(hwnd),
            None,
            None,
            None,
        )?;

        if button_hwnd.is_invalid() {
            return Err(Error::from_thread());
        }

        ShowWindow(hwnd, SW_SHOW);

        // Message loop
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Ok(())
    }
}
