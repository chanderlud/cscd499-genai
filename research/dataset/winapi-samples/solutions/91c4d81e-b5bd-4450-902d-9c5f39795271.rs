use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{GetSysColorBrush, COLOR_WINDOW};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class_name = wide_null("ListBoxExample");

        let wc = WNDCLASSW {
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hInstance: instance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            hbrBackground: GetSysColorBrush(COLOR_WINDOW),
            ..Default::default()
        };

        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(wide_null("ListBox Clear and Repopulate Example").as_ptr()),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        let listbox = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(wide_null("ListBox").as_ptr()),
            PCWSTR::null(),
            WS_CHILD | WS_VISIBLE | WS_VSCROLL | WS_BORDER | WINDOW_STYLE(LBS_NOTIFY as u32),
            10,
            10,
            200,
            200,
            Some(hwnd),
            None,
            Some(instance.into()),
            None,
        )?;

        // Initial population
        for i in 0..5 {
            let text = format!("Initial Item {}", i);
            SendMessageW(
                listbox,
                LB_ADDSTRING,
                None,
                Some(LPARAM(wide_null(&text).as_ptr() as isize)),
            );
        }

        // Clear and repopulate with storage optimization
        SendMessageW(listbox, LB_RESETCONTENT, None, None);

        // Pre-allocate storage for 10 items, each about 20 characters
        SendMessageW(
            listbox,
            LB_INITSTORAGE,
            Some(WPARAM(10)),
            Some(LPARAM(20 * 10)),
        );

        // Add new items
        for i in 0..10 {
            let text = format!("Repopulated Item {}", i);
            SendMessageW(
                listbox,
                LB_ADDSTRING,
                None,
                Some(LPARAM(wide_null(&text).as_ptr() as isize)),
            );
        }

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).into() {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }

        Ok(())
    }
}

extern "system" fn wndproc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, message, wparam, lparam),
        }
    }
}
