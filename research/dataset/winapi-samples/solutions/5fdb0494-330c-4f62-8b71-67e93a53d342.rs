use std::mem::zeroed;
use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, MessageBoxW, PostQuitMessage,
    RegisterClassExW, SendMessageW, ShowWindow, TranslateMessage, CW_USEDEFAULT, LBS_HASSTRINGS,
    LBS_MULTIPLESEL, LBS_NOTIFY, LB_ADDSTRING, LB_GETSELCOUNT, LB_SETSEL, MSG, SW_SHOWDEFAULT,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_CREATE, WM_DESTROY, WNDCLASSEXW, WS_BORDER, WS_CHILD,
    WS_VISIBLE, WS_VSCROLL,
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
            hInstance: HINSTANCE(instance.0),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..zeroed()
        };

        RegisterClassExW(&wc);

        let window_name = wide_null("ListBox Selection Count Example");
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_name.as_ptr()),
            WINDOW_STYLE(WS_VISIBLE.0),
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            Some(HINSTANCE(instance.0)),
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
                let listbox_class = wide_null("ListBox");
                let listbox_hwnd = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    PCWSTR(listbox_class.as_ptr()),
                    PCWSTR::null(),
                    WINDOW_STYLE(
                        WS_CHILD.0
                            | WS_VISIBLE.0
                            | WS_BORDER.0
                            | WS_VSCROLL.0
                            | LBS_NOTIFY as u32
                            | LBS_HASSTRINGS as u32
                            | LBS_MULTIPLESEL as u32,
                    ),
                    10,
                    10,
                    200,
                    200,
                    Some(hwnd),
                    None,
                    None,
                    None,
                )
                .unwrap();

                // Add some items
                let items = ["Apple", "Banana", "Cherry", "Date", "Elderberry"];
                for item in items {
                    let wide_item = wide_null(item);
                    SendMessageW(
                        listbox_hwnd,
                        LB_ADDSTRING,
                        Some(WPARAM(0)),
                        Some(LPARAM(wide_item.as_ptr() as isize)),
                    );
                }

                // Select some items (indices 0, 2, and 4)
                for index in [0, 2, 4] {
                    SendMessageW(
                        listbox_hwnd,
                        LB_SETSEL,
                        Some(WPARAM(1)),
                        Some(LPARAM(index)),
                    );
                }

                // Get the number of selected items
                let selected_count = SendMessageW(
                    listbox_hwnd,
                    LB_GETSELCOUNT,
                    Some(WPARAM(0)),
                    Some(LPARAM(0)),
                )
                .0 as i32;

                // Display the result
                let message = format!("Number of selected items: {}", selected_count);
                let wide_message = wide_null(&message);
                MessageBoxW(
                    Some(hwnd),
                    PCWSTR(wide_message.as_ptr()),
                    PCWSTR(wide_null("Selection Count").as_ptr()),
                    Default::default(),
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
