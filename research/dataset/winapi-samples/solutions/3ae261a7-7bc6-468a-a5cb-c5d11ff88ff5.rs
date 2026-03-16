use std::ffi::OsStr;
use std::ptr;
use windows::core::{Error, Result};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassW, SendMessageW, ShowWindow, TranslateMessage, CW_USEDEFAULT, LBS_HASSTRINGS,
    LBS_NOTIFY, LB_ADDSTRING, LB_ERR, LB_GETCURSEL, LB_GETTEXT, LB_GETTEXTLEN, LB_SETCURSEL, MSG,
    SW_SHOW, WINDOW_STYLE, WM_DESTROY, WNDCLASSW, WS_BORDER, WS_CHILD, WS_OVERLAPPEDWINDOW,
    WS_VISIBLE, WS_VSCROLL,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        // Register window class
        let class_name = wide_null(OsStr::new("ListBoxExample"));
        let wnd_class = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: Default::default(),
            lpszClassName: windows::core::PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        if RegisterClassW(&wnd_class) == 0 {
            return Err(Error::from_thread());
        }

        // Create main window
        let window_title = wide_null(OsStr::new("ListBox Example"));
        let hwnd = CreateWindowExW(
            Default::default(),
            windows::core::PCWSTR(class_name.as_ptr()),
            windows::core::PCWSTR(window_title.as_ptr()),
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

        // Create ListBox control
        let listbox_class = wide_null(OsStr::new("ListBox"));
        let listbox_hwnd = CreateWindowExW(
            Default::default(),
            windows::core::PCWSTR(listbox_class.as_ptr()),
            windows::core::PCWSTR::null(),
            WS_CHILD
                | WS_VISIBLE
                | WS_BORDER
                | WS_VSCROLL
                | WINDOW_STYLE(LBS_HASSTRINGS as u32)
                | WINDOW_STYLE(LBS_NOTIFY as u32),
            10,
            10,
            200,
            200,
            Some(hwnd),
            None,
            None,
            None,
        )?;

        // Add items to ListBox
        let items = ["Apple", "Banana", "Cherry", "Date", "Elderberry"];
        for item in items.iter() {
            let wide_item = wide_null(OsStr::new(item));
            let result = SendMessageW(
                listbox_hwnd,
                LB_ADDSTRING,
                Some(WPARAM(0)),
                Some(LPARAM(wide_item.as_ptr() as isize)),
            );

            if result.0 == LB_ERR as isize {
                return Err(Error::from_hresult(windows::core::HRESULT::from_win32(
                    windows::Win32::Foundation::ERROR_INVALID_INDEX.0,
                )));
            }
        }

        // Select third item (index 2)
        let result = SendMessageW(listbox_hwnd, LB_SETCURSEL, Some(WPARAM(2)), Some(LPARAM(0)));

        if result.0 == LB_ERR as isize {
            return Err(Error::from_hresult(windows::core::HRESULT::from_win32(
                windows::Win32::Foundation::ERROR_INVALID_INDEX.0,
            )));
        }

        // Get selected index
        let selected_index =
            SendMessageW(listbox_hwnd, LB_GETCURSEL, Some(WPARAM(0)), Some(LPARAM(0)));

        if selected_index.0 == LB_ERR as isize {
            println!("No item selected");
        } else {
            // Get text length
            let text_len = SendMessageW(
                listbox_hwnd,
                LB_GETTEXTLEN,
                Some(WPARAM(selected_index.0 as usize)),
                Some(LPARAM(0)),
            );

            if text_len.0 == LB_ERR as isize {
                return Err(Error::from_hresult(windows::core::HRESULT::from_win32(
                    windows::Win32::Foundation::ERROR_INVALID_INDEX.0,
                )));
            }

            // Allocate buffer and get text
            let mut buffer: Vec<u16> = vec![0; (text_len.0 as usize) + 1];
            let result = SendMessageW(
                listbox_hwnd,
                LB_GETTEXT,
                Some(WPARAM(selected_index.0 as usize)),
                Some(LPARAM(buffer.as_mut_ptr() as isize)),
            );

            if result.0 == LB_ERR as isize {
                return Err(Error::from_hresult(windows::core::HRESULT::from_win32(
                    windows::Win32::Foundation::ERROR_INVALID_INDEX.0,
                )));
            }

            // Convert to String and print
            let selected_string = String::from_utf16_lossy(&buffer[..text_len.0 as usize]);
            println!("Selected item: {}", selected_string);
        }

        // Show window and run message loop
        ShowWindow(hwnd, SW_SHOW);

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
