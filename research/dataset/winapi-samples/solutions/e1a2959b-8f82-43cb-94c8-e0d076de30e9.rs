use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetFocus, SetFocus};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassW, ShowWindow, TranslateMessage, CW_USEDEFAULT, MSG, SW_SHOW, WINDOW_EX_STYLE,
    WINDOW_STYLE, WM_DESTROY, WNDCLASSW, WS_CHILD, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class_name = wide_null("FocusExampleClass".as_ref());

        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        RegisterClassW(&wc);

        let window_title = wide_null("Button Focus Example".as_ref());
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
            WS_CHILD | WS_VISIBLE,
            50,
            50,
            100,
            30,
            Some(hwnd),
            None,
            Some(instance.into()),
            None,
        )?;

        // Set focus to the button
        SetFocus(Some(button_hwnd));

        // Check if the button has focus
        let focused_hwnd = GetFocus();
        if focused_hwnd == button_hwnd {
            println!("Button successfully received focus");
        } else {
            println!("Button did not receive focus");
        }

        ShowWindow(hwnd, SW_SHOW);

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
