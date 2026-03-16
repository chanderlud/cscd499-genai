use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassW, SendMessageW,
    TranslateMessage, CW_USEDEFAULT, LBS_HASSTRINGS, LBS_NOTIFY, LB_INSERTSTRING, MSG,
    WINDOW_STYLE, WNDCLASSW, WS_BORDER, WS_CHILD, WS_OVERLAPPEDWINDOW, WS_VISIBLE, WS_VSCROLL,
};

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let class_name = wide_null("ListBoxInsertExample");
        let window_title = wide_null("Insert Item Example");

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
        )?;

        let listbox_class = wide_null("ListBox");
        let listbox = CreateWindowExW(
            Default::default(),
            PCWSTR(listbox_class.as_ptr()),
            PCWSTR::null(),
            WS_CHILD
                | WS_VISIBLE
                | WS_BORDER
                | WS_VSCROLL
                | WINDOW_STYLE((LBS_HASSTRINGS | LBS_NOTIFY) as u32),
            10,
            10,
            360,
            200,
            Some(hwnd),
            None,
            None,
            None,
        )?;

        // Add initial items
        let items = ["First", "Second", "Fourth"];
        for item in items {
            let wide = wide_null(item);
            let index = SendMessageW(
                listbox,
                LB_INSERTSTRING,
                Some(WPARAM(usize::MAX)),
                Some(LPARAM(wide.as_ptr() as isize)),
            );
            if index.0 < 0 {
                return Err(Error::from_hresult(windows::core::HRESULT(index.0 as i32)));
            }
        }

        // Insert "Third" at index 2 (between "Second" and "Fourth")
        let third_item = wide_null("Third");
        let index = SendMessageW(
            listbox,
            LB_INSERTSTRING,
            Some(WPARAM(2)),
            Some(LPARAM(third_item.as_ptr() as isize)),
        );
        if index.0 < 0 {
            return Err(Error::from_hresult(windows::core::HRESULT(index.0 as i32)));
        }

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).into() {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }

        Ok(())
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    DefWindowProcW(hwnd, msg, wparam, lparam)
}
