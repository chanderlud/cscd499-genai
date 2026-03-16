use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, LoadIconW, PostQuitMessage,
    RegisterClassW, SendMessageW, TranslateMessage, BM_SETIMAGE, BS_ICON, BS_NOTIFY,
    IDI_APPLICATION, IMAGE_ICON, MSG, WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY, WNDCLASSW,
    WS_CHILD, WS_OVERLAPPEDWINDOW, WS_TABSTOP, WS_VISIBLE,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let hinstance = HINSTANCE(instance.0);
        let class_name = wide_null("ButtonIconExample".as_ref());

        let wnd_class = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: hinstance,
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        RegisterClassW(&wnd_class);

        let window_title = wide_null("Icon Button Example".as_ref());
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_title.as_ptr()),
            WINDOW_STYLE(WS_OVERLAPPEDWINDOW.0 | WS_VISIBLE.0),
            100,
            100,
            400,
            300,
            None,
            None,
            Some(hinstance),
            None,
        )?;

        let button_text = wide_null("Button with Icon".as_ref());
        let button_class = wide_null("BUTTON".as_ref());
        let button_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(button_class.as_ptr()),
            PCWSTR(button_text.as_ptr()),
            WINDOW_STYLE(
                WS_VISIBLE.0 | WS_CHILD.0 | WS_TABSTOP.0 | BS_NOTIFY as u32 | BS_ICON as u32,
            ),
            50,
            50,
            200,
            50,
            Some(hwnd),
            None,
            Some(hinstance),
            None,
        )?;

        // Load a system icon
        let icon = LoadIconW(None, IDI_APPLICATION)?;

        // Set the icon on the button
        SendMessageW(
            button_hwnd,
            BM_SETIMAGE,
            Some(WPARAM(IMAGE_ICON.0 as usize)),
            Some(LPARAM(icon.0 as isize)),
        );

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
