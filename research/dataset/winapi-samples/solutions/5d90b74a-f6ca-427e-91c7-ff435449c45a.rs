use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassExW, SendMessageW, TranslateMessage, CW_USEDEFAULT, LBS_HASSTRINGS,
    LBS_MULTIPLESEL, LBS_NOTIFY, LB_ADDSTRING, LB_GETCOUNT, LB_RESETCONTENT, MSG, WINDOW_EX_STYLE,
    WINDOW_STYLE, WM_DESTROY, WNDCLASSEXW, WS_BORDER, WS_CHILD, WS_TABSTOP, WS_VISIBLE, WS_VSCROLL,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;

        let class_name = wide_null("ListBoxClearExample".as_ref());
        let window_title = wide_null("Clear ListBox Example".as_ref());

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        let atom = RegisterClassExW(&wc);
        if atom == 0 {
            return Err(windows::core::Error::from_thread());
        }

        let style = WS_VISIBLE
            | WS_CHILD
            | WS_VSCROLL
            | WS_BORDER
            | WS_TABSTOP
            | WINDOW_STYLE(LBS_HASSTRINGS as u32)
            | WINDOW_STYLE(LBS_NOTIFY as u32)
            | WINDOW_STYLE(LBS_MULTIPLESEL as u32);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_title.as_ptr()),
            style,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        // Add some items to the ListBox
        let items = ["Item 1", "Item 2", "Item 3", "Item 4", "Item 5"];
        for item in items {
            let wide_item = wide_null(item.as_ref());
            SendMessageW(
                hwnd,
                LB_ADDSTRING,
                Some(WPARAM(0)),
                Some(LPARAM(wide_item.as_ptr() as _)),
            );
        }

        // Verify items were added
        let count_before =
            SendMessageW(hwnd, LB_GETCOUNT, Some(WPARAM(0)), Some(LPARAM(0))).0 as i32;
        println!("Items before clear: {}", count_before);

        // Clear all items using LB_RESETCONTENT
        SendMessageW(hwnd, LB_RESETCONTENT, Some(WPARAM(0)), Some(LPARAM(0)));

        // Verify items were cleared
        let count_after =
            SendMessageW(hwnd, LB_GETCOUNT, Some(WPARAM(0)), Some(LPARAM(0))).0 as i32;
        println!("Items after clear: {}", count_after);

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
