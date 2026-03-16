// TITLE: Set and get selection range in an edit control

use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::{EM_GETSEL, EM_SETSEL};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassW, SendMessageW, TranslateMessage, ES_AUTOHSCROLL, ES_LEFT, MSG, WINDOW_EX_STYLE,
    WINDOW_STYLE, WM_DESTROY, WNDCLASSW, WS_BORDER, WS_CHILD, WS_VISIBLE,
};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let instance: HINSTANCE = GetModuleHandleW(None)?.into();
        let class_name = wide_null(OsStr::new("SelectionExampleWindowClass"));

        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        RegisterClassW(&wc);

        let window_title = wide_null(OsStr::new("Edit Control Selection Example"));
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_title.as_ptr()),
            WS_VISIBLE,
            100,
            100,
            400,
            300,
            None,
            None,
            Some(instance),
            None,
        )?;

        let edit_text = wide_null(OsStr::new(
            "Hello, this is some sample text for selection testing.",
        ));
        let edit_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(wide_null(OsStr::new("EDIT")).as_ptr()),
            PCWSTR(edit_text.as_ptr()),
            WINDOW_STYLE(
                WS_VISIBLE.0 | WS_CHILD.0 | WS_BORDER.0 | ES_AUTOHSCROLL as u32 | ES_LEFT as u32,
            ),
            20,
            20,
            350,
            30,
            Some(hwnd),
            None,
            Some(instance),
            None,
        )?;

        // Set selection to first 5 characters (0 to 5)
        let result = SendMessageW(edit_hwnd, EM_SETSEL, Some(WPARAM(0)), Some(LPARAM(5)));
        if result == LRESULT(0) {
            return Err(Error::from_thread());
        }

        // Get current selection
        let mut start: i32 = 0;
        let mut end: i32 = 0;
        let result = SendMessageW(
            edit_hwnd,
            EM_GETSEL,
            Some(WPARAM(&mut start as *mut _ as usize)),
            Some(LPARAM(&mut end as *mut _ as isize)),
        );

        if result == LRESULT(0) {
            return Err(Error::from_thread());
        }

        println!("Selection range: {} to {}", start, end);

        // Change selection to characters 10-15
        let result = SendMessageW(edit_hwnd, EM_SETSEL, Some(WPARAM(10)), Some(LPARAM(15)));
        if result == LRESULT(0) {
            return Err(Error::from_thread());
        }

        // Get updated selection
        let mut start2: i32 = 0;
        let mut end2: i32 = 0;
        let result = SendMessageW(
            edit_hwnd,
            EM_GETSEL,
            Some(WPARAM(&mut start2 as *mut _ as usize)),
            Some(LPARAM(&mut end2 as *mut _ as isize)),
        );

        if result == LRESULT(0) {
            return Err(Error::from_thread());
        }

        println!("Updated selection range: {} to {}", start2, end2);

        // Select all text
        let result = SendMessageW(edit_hwnd, EM_SETSEL, Some(WPARAM(0)), Some(LPARAM(-1)));
        if result == LRESULT(0) {
            return Err(Error::from_thread());
        }

        // Get full selection
        let mut start3: i32 = 0;
        let mut end3: i32 = 0;
        let result = SendMessageW(
            edit_hwnd,
            EM_GETSEL,
            Some(WPARAM(&mut start3 as *mut _ as usize)),
            Some(LPARAM(&mut end3 as *mut _ as isize)),
        );

        if result == LRESULT(0) {
            return Err(Error::from_thread());
        }

        println!("Full selection range: {} to {}", start3, end3);

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
