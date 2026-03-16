use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{COLOR_WINDOW, HBRUSH};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, LoadCursorW, PostQuitMessage,
    RegisterClassExW, SendMessageW, ShowWindow, TranslateMessage, CS_HREDRAW, CS_VREDRAW,
    CW_USEDEFAULT, HCURSOR, HICON, IDC_ARROW, LBS_HASSTRINGS, LBS_MULTIPLESEL, LBS_NOTIFY,
    LB_ADDSTRING, LB_GETSEL, LB_SETSEL, MSG, SW_SHOW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY,
    WNDCLASSEXW, WS_BORDER, WS_CHILD, WS_OVERLAPPEDWINDOW, WS_VISIBLE, WS_VSCROLL,
};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let instance: HINSTANCE = GetModuleHandleW(None)?.into();
        let class_name = wide_null(OsStr::new("ListBoxCheckExample"));

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance.into(),
            hIcon: HICON::default(),
            hCursor: load_cursor_w(None, IDC_ARROW)?,
            hbrBackground: HBRUSH((COLOR_WINDOW.0 + 1) as *mut _),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            hIconSm: HICON::default(),
        };

        let atom = RegisterClassExW(&wc);
        if atom == 0 {
            return Err(windows::core::Error::from_thread());
        }

        let window_title = wide_null(OsStr::new("ListBox Selection Check Example"));
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_title.as_ptr()),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            Some(instance),
            None,
        )?;

        // Create a multi-select ListBox
        let listbox_class = wide_null(OsStr::new("ListBox"));
        let listbox = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(listbox_class.as_ptr()),
            PCWSTR::null(),
            WS_CHILD
                | WS_VISIBLE
                | WS_BORDER
                | WS_VSCROLL
                | WINDOW_STYLE(LBS_MULTIPLESEL as u32 | LBS_HASSTRINGS as u32 | LBS_NOTIFY as u32),
            10,
            10,
            200,
            200,
            Some(hwnd),
            None,
            Some(instance),
            None,
        )?;

        // Add some items to the ListBox
        let items = ["Apple", "Banana", "Cherry", "Date", "Elderberry"];
        for item in items.iter() {
            let wide_item = wide_null(OsStr::new(item));
            send_message_w(
                listbox,
                LB_ADDSTRING,
                WPARAM(0),
                LPARAM(wide_item.as_ptr() as isize),
            );
        }

        // Select items at indices 0, 2, and 4 (Apple, Cherry, Elderberry)
        send_message_w(listbox, LB_SETSEL, WPARAM(1), LPARAM(0));
        send_message_w(listbox, LB_SETSEL, WPARAM(1), LPARAM(2));
        send_message_w(listbox, LB_SETSEL, WPARAM(1), LPARAM(4));

        // Check which items are selected using LB_GETSEL
        println!("Checking selection status of ListBox items:");
        for i in 0..items.len() {
            let result = send_message_w(listbox, LB_GETSEL, WPARAM(i), LPARAM(0));
            let is_selected = result.0 > 0;
            println!(
                "Item '{}' (index {}): {}",
                items[i],
                i,
                if is_selected {
                    "SELECTED"
                } else {
                    "not selected"
                }
            );
        }

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

unsafe fn send_message_w(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    SendMessageW(hwnd, msg, Some(wparam), Some(lparam))
}

unsafe fn load_cursor_w(instance: Option<HINSTANCE>, name: PCWSTR) -> Result<HCURSOR> {
    LoadCursorW(instance, name)
}
