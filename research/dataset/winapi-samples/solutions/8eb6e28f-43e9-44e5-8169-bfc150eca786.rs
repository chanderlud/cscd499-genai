use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{E_FAIL, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{COLOR_WINDOW, HBRUSH};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class_name = wide_null("SampleButtonClass".as_ref());

        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            hbrBackground: HBRUSH((COLOR_WINDOW.0 + 1) as *mut core::ffi::c_void),
            ..Default::default()
        };

        // RegisterClassW returns 0 on failure
        if RegisterClassW(&wc) == 0 {
            return Err(Error::from_thread());
        }

        let window_title = wide_null("Button Click Example".as_ref());
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
            Some(instance.into()),
            None,
        )?;

        let button_text = wide_null("Click Me".as_ref());
        let button_class = wide_null("BUTTON".as_ref());
        let button_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(button_class.as_ptr()),
            PCWSTR(button_text.as_ptr()),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_PUSHBUTTON as u32),
            50,
            50,
            120,
            30,
            Some(hwnd),
            None,
            Some(instance.into()),
            None,
        )?;

        // Simulate a button click programmatically
        let result = SendMessageW(button_hwnd, BM_CLICK, Some(WPARAM(0)), Some(LPARAM(0)));
        if result == LRESULT(0) {
            // BM_CLICK returns 0 on success
            println!("Button click simulated successfully");
        } else {
            // SendMessageW doesn't set GetLastError, so we return a generic error
            return Err(Error::new(E_FAIL, "SendMessageW failed"));
        }

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).into() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }

        Ok(())
    }
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_COMMAND => {
                // Check if the message is from our button (BN_CLICKED)
                let _control_id = (wparam.0 & 0xFFFF) as u16;
                let notification_code = (wparam.0 >> 16) as u16;

                if notification_code == BN_CLICKED as u16 {
                    println!("Button was clicked (via BN_CLICKED notification)");
                    return LRESULT(0);
                }
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                return LRESULT(0);
            }
            _ => {}
        }
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}
