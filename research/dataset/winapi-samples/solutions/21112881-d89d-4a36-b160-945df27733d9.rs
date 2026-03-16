// TITLE: Set and retrieve text limit in an edit control

use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Controls::{EM_GETLIMITTEXT, EM_SETLIMITTEXT};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassW, SendMessageW, SetWindowLongPtrW, TranslateMessage, UnregisterClassW,
    CREATESTRUCTW, CW_USEDEFAULT, ES_AUTOHSCROLL, GWLP_USERDATA, MSG, WINDOW_STYLE, WM_CREATE,
    WM_DESTROY, WNDCLASSW, WS_BORDER, WS_CHILD, WS_OVERLAPPEDWINDOW, WS_TABSTOP, WS_VISIBLE,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let class_name = wide_null(OsStr::new("SampleEditClass"));
        let window_title = wide_null(OsStr::new("Edit Control Text Limit Example"));

        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: Default::default(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        let atom = RegisterClassW(&wc);
        if atom == 0 {
            return Err(Error::from_thread());
        }

        let hwnd = CreateWindowExW(
            Default::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_title.as_ptr()),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            None,
            None,
        );

        let hwnd = match hwnd {
            Ok(hwnd) => hwnd,
            Err(e) => {
                UnregisterClassW(PCWSTR(class_name.as_ptr()), None);
                return Err(e);
            }
        };

        if hwnd.0 == std::ptr::null_mut() {
            UnregisterClassW(PCWSTR(class_name.as_ptr()), None);
            return Err(Error::from_thread());
        }

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        UnregisterClassW(PCWSTR(class_name.as_ptr()), None);
        Ok(())
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            let cs = lparam.0 as *const CREATESTRUCTW;
            let edit_style = WINDOW_STYLE(
                WS_CHILD.0 | WS_VISIBLE.0 | WS_BORDER.0 | (ES_AUTOHSCROLL as u32) | WS_TABSTOP.0,
            );

            let edit_hwnd = CreateWindowExW(
                Default::default(),
                PCWSTR(wide_null(OsStr::new("EDIT")).as_ptr()),
                PCWSTR(wide_null(OsStr::new("Initial text")).as_ptr()),
                edit_style,
                50,
                50,
                200,
                25,
                Some(hwnd),
                None,
                None,
                None,
            );

            let edit_hwnd = match edit_hwnd {
                Ok(hwnd) => hwnd,
                Err(_) => return LRESULT(-1),
            };

            // Set text limit to 10 characters
            SendMessageW(
                edit_hwnd,
                EM_SETLIMITTEXT,
                Some(WPARAM(10)),
                Some(LPARAM(0)),
            );

            // Retrieve the text limit
            let limit = SendMessageW(edit_hwnd, EM_GETLIMITTEXT, None, None);
            println!("Text limit set to: {}", limit.0);

            // Store edit control handle in window user data
            let _ = SetWindowLongPtrW(hwnd, GWLP_USERDATA, edit_hwnd.0 as isize);
        }
        WM_DESTROY => {
            PostQuitMessage(0);
        }
        _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
    }
    LRESULT(0)
}
