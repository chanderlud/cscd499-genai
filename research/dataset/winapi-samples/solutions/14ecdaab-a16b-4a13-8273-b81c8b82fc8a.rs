use std::mem;
use std::ptr;
use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassExW, ShowWindow, TranslateMessage, CW_USEDEFAULT, LBS_HASSTRINGS, LBS_NOTIFY,
    LB_ADDSTRING, LB_ERR, LB_GETCURSEL, LB_GETTEXT, LB_GETTEXTLEN, LB_SETCURSEL, MSG, SW_SHOW,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_CREATE, WM_DESTROY, WNDCLASSEXW, WS_BORDER, WS_CHILD,
    WS_VISIBLE, WS_VSCROLL,
};

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let instance: HINSTANCE = GetModuleHandleW(None)?.into();
        let class_name = wide_null("ListBoxExample");

        let wc = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        let atom = RegisterClassExW(&wc);
        if atom == 0 {
            return Err(windows::core::Error::from_thread());
        }

        let window_name = wide_null("ListBox Text Example");
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
            Some(instance),
            None,
        )?;

        ShowWindow(hwnd, SW_SHOW);

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
                // Create ListBox
                let listbox_name = wide_null("ListBox");
                let listbox = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    PCWSTR(listbox_name.as_ptr()),
                    PCWSTR(ptr::null()),
                    WINDOW_STYLE(
                        WS_CHILD.0
                            | WS_VISIBLE.0
                            | WS_BORDER.0
                            | WS_VSCROLL.0
                            | LBS_NOTIFY as u32
                            | LBS_HASSTRINGS as u32,
                    ),
                    10,
                    10,
                    200,
                    200,
                    Some(hwnd),
                    None,
                    Some(GetModuleHandleW(None).unwrap().into()),
                    None,
                )
                .unwrap();

                // Add items
                let items = ["Apple", "Banana", "Cherry", "Date", "Elderberry"];
                for item in items {
                    let wide_item = wide_null(item);
                    windows::Win32::UI::WindowsAndMessaging::SendMessageW(
                        listbox,
                        LB_ADDSTRING,
                        Some(WPARAM(0)),
                        Some(LPARAM(wide_item.as_ptr() as isize)),
                    );
                }

                // Select second item (index 1)
                windows::Win32::UI::WindowsAndMessaging::SendMessageW(
                    listbox,
                    LB_SETCURSEL,
                    Some(WPARAM(1)),
                    Some(LPARAM(0)),
                );

                // Get selected index
                let selected_index = windows::Win32::UI::WindowsAndMessaging::SendMessageW(
                    listbox,
                    LB_GETCURSEL,
                    Some(WPARAM(0)),
                    Some(LPARAM(0)),
                );

                if selected_index.0 != LB_ERR as isize {
                    // Get text length
                    let text_len = windows::Win32::UI::WindowsAndMessaging::SendMessageW(
                        listbox,
                        LB_GETTEXTLEN,
                        Some(WPARAM(selected_index.0 as usize)),
                        Some(LPARAM(0)),
                    );

                    if text_len.0 > 0 {
                        // Allocate buffer for text
                        let mut buffer = vec![0u16; (text_len.0 + 1) as usize];

                        // Get text
                        windows::Win32::UI::WindowsAndMessaging::SendMessageW(
                            listbox,
                            LB_GETTEXT,
                            Some(WPARAM(selected_index.0 as usize)),
                            Some(LPARAM(buffer.as_mut_ptr() as isize)),
                        );

                        // Convert to String and print
                        let selected_text =
                            String::from_utf16_lossy(&buffer[..text_len.0 as usize]);
                        println!("Selected item: {}", selected_text);
                    }
                }

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
