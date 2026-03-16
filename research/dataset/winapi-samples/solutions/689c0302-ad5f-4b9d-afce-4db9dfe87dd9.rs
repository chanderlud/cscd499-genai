use std::mem;
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{E_FAIL, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{GetStockObject, HBRUSH, WHITE_BRUSH};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassExW, SendMessageW, TranslateMessage, CW_USEDEFAULT, LBS_HASSTRINGS,
    LBS_NOINTEGRALHEIGHT, LBS_NOTIFY, LB_ADDSTRING, LB_DELETESTRING, LB_ERR, MSG, WINDOW_EX_STYLE,
    WINDOW_STYLE, WM_DESTROY, WNDCLASSEXW, WS_BORDER, WS_CHILD, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
    WS_VSCROLL,
};

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let class_name = wide_null("ListBoxExample");
        let window_title = wide_null("ListBox Remove Item Example");

        let wc = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wndproc),
            hInstance: Default::default(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            hbrBackground: {
                // GetStockObject returns HGDIOBJ, not Result, so we can't use ?
                let hbr = GetStockObject(WHITE_BRUSH);
                if hbr.0.is_null() {
                    return Err(Error::new(E_FAIL, "Failed to get stock brush"));
                }
                HBRUSH(hbr.0)
            },
            ..Default::default()
        };

        RegisterClassExW(&wc);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_title.as_ptr()),
            WINDOW_STYLE(WS_VISIBLE.0 | WS_OVERLAPPEDWINDOW.0),
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            None,
            None,
        )?;

        let listbox = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(wide_null("ListBox").as_ptr()),
            PCWSTR::null(),
            WINDOW_STYLE(
                WS_VISIBLE.0
                    | WS_CHILD.0
                    | WS_VSCROLL.0
                    | WS_BORDER.0
                    | LBS_HASSTRINGS as u32
                    | LBS_NOTIFY as u32
                    | LBS_NOINTEGRALHEIGHT as u32,
            ),
            10,
            10,
            200,
            200,
            Some(hwnd),
            None,
            None,
            None,
        )?;

        // Add items to the listbox
        let items = ["First Item", "Second Item", "Third Item", "Fourth Item"];
        for item in items {
            let wide_item = wide_null(item);
            let result = SendMessageW(
                listbox,
                LB_ADDSTRING,
                None,
                Some(LPARAM(wide_item.as_ptr() as isize)),
            );
            if result.0 == LB_ERR as isize {
                return Err(Error::new(E_FAIL, "Failed to add string to list box"));
            }
        }

        // Remove the second item (index 1) from the listbox
        let remove_index = 1;
        let result = SendMessageW(
            listbox,
            LB_DELETESTRING,
            Some(WPARAM(remove_index)),
            Some(LPARAM(0)),
        );
        if result.0 == LB_ERR as isize {
            return Err(Error::new(E_FAIL, "Failed to delete string from list box"));
        }

        println!("Successfully removed item at index {}", remove_index);

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
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
