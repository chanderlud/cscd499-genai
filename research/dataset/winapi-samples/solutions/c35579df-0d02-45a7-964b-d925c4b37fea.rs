use std::ffi::OsStr;
use std::iter::once;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassExW, SendMessageW, ShowWindow, TranslateMessage, CW_USEDEFAULT, LBS_HASSTRINGS,
    LBS_MULTIPLESEL, LBS_NOINTEGRALHEIGHT, LBS_NOTIFY, LB_ADDSTRING, LB_SELITEMRANGEEX, MSG,
    SW_SHOWDEFAULT, WINDOW_EX_STYLE, WINDOW_STYLE, WM_CREATE, WM_DESTROY, WNDCLASSEXW, WS_BORDER,
    WS_CHILD, WS_VISIBLE, WS_VSCROLL,
};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class_name = wide_null(OsStr::new("ListBoxRangeExample"));

        let wc = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        RegisterClassExW(&wc);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(wide_null(OsStr::new("ListBox Range Selection Example")).as_ptr()),
            WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        ShowWindow(hwnd, SW_SHOWDEFAULT);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Ok(())
    }
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => {
                // Create a multi-select ListBox
                let style_flags = WS_CHILD.0
                    | WS_VISIBLE.0
                    | WS_BORDER.0
                    | WS_VSCROLL.0
                    | LBS_NOTIFY as u32
                    | LBS_HASSTRINGS as u32
                    | LBS_MULTIPLESEL as u32
                    | LBS_NOINTEGRALHEIGHT as u32;

                let listbox = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    PCWSTR(wide_null(OsStr::new("ListBox")).as_ptr()),
                    PCWSTR::null(),
                    WINDOW_STYLE(style_flags),
                    10,
                    10,
                    360,
                    200,
                    Some(hwnd),
                    None,
                    None,
                    None,
                )
                .unwrap();

                // Add some items to the ListBox
                for i in 1..=10 {
                    let text = format!("Item {}", i);
                    let wide_text = wide_null(OsStr::new(&text));
                    SendMessageW(
                        listbox,
                        LB_ADDSTRING,
                        Some(WPARAM(0)),
                        Some(LPARAM(wide_text.as_ptr() as isize)),
                    );
                }

                // Select a range of items (indices 2 to 5, inclusive)
                // LB_SELITEMRANGEEX selects items from start (wparam) to end (lparam)
                // Note: This message selects the range, it doesn't toggle
                SendMessageW(
                    listbox,
                    LB_SELITEMRANGEEX,
                    Some(WPARAM(2)), // Start index (0-based)
                    Some(LPARAM(5)), // End index (0-based, inclusive)
                );

                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
