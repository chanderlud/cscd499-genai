// TITLE: Set text and get length of an edit control without string allocation

use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::EM_LINELENGTH;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassW, SendMessageW, TranslateMessage, ES_AUTOHSCROLL, MSG, WINDOW_EX_STYLE,
    WINDOW_STYLE, WM_DESTROY, WM_SETTEXT, WNDCLASSW, WS_BORDER, WS_CHILD, WS_OVERLAPPEDWINDOW,
    WS_VISIBLE,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class_name = wide_null("SampleEditClass".as_ref());

        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        let atom = RegisterClassW(&wc);
        if atom == 0 {
            return Err(Error::from_thread());
        }

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(wide_null("Edit Control Example".as_ref()).as_ptr()),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            100,
            100,
            400,
            300,
            None,
            None,
            Some(HINSTANCE(instance.0)),
            None,
        )?;

        let edit_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(wide_null("EDIT".as_ref()).as_ptr()),
            PCWSTR::null(),
            WS_CHILD | WS_VISIBLE | WS_BORDER | WINDOW_STYLE(ES_AUTOHSCROLL as u32),
            50,
            50,
            200,
            25,
            Some(hwnd),
            None,
            Some(HINSTANCE(instance.0)),
            None,
        )?;

        // Set text in the edit control
        let text = wide_null("Hello, World!".as_ref());
        let result = SendMessageW(
            edit_hwnd,
            WM_SETTEXT,
            Some(WPARAM(0)),
            Some(LPARAM(text.as_ptr() as isize)),
        );
        if result == LRESULT(0) {
            return Err(Error::from_thread());
        }

        // Get length of text without string allocation using EM_LINELENGTH
        let length = SendMessageW(edit_hwnd, EM_LINELENGTH, Some(WPARAM(0)), Some(LPARAM(0)));
        println!("Text length: {}", length.0);

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).into() {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }

        Ok(())
    }
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
