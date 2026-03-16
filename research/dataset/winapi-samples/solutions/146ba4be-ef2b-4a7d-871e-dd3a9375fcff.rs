use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassExW, SendMessageW, ShowWindow, TranslateMessage, CW_USEDEFAULT, LBS_HASSTRINGS,
    LBS_NOTIFY, LB_ADDSTRING, MSG, SW_SHOWDEFAULT, WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY,
    WNDCLASSEXW, WS_BORDER, WS_CHILD, WS_OVERLAPPEDWINDOW, WS_VSCROLL,
};

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class_name = wide_null("ListBoxExample");

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        RegisterClassExW(&wc);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(wide_null("ListBox Example").as_ptr()),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        // Create ListBox control
        let listbox_class = wide_null("ListBox");
        let listbox = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(listbox_class.as_ptr()),
            PCWSTR::null(),
            WINDOW_STYLE(
                LBS_HASSTRINGS as u32 | LBS_NOTIFY as u32 | WS_BORDER.0 | WS_VSCROLL.0 | WS_CHILD.0,
            ),
            10,
            10,
            360,
            200,
            Some(hwnd),
            None,
            Some(instance.into()),
            None,
        )?;

        // Add items to ListBox using LB_ADDSTRING
        let items = ["Item 1", "Item 2", "Item 3", "Item 4", "Item 5"];
        for item in items {
            let wide_item = wide_null(item);
            let result = SendMessageW(
                listbox,
                LB_ADDSTRING,
                Some(WPARAM(0)),
                Some(LPARAM(wide_item.as_ptr() as isize)),
            );

            if result.0 == -1 {
                return Err(Error::from_thread());
            }
        }

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
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
