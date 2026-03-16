use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::{EM_SETREADONLY, WC_EDITW};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, GetWindowLongW,
    PostQuitMessage, RegisterClassExW, SendMessageW, TranslateMessage, CW_USEDEFAULT,
    ES_AUTOHSCROLL, ES_READONLY, GWL_STYLE, MSG, WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY,
    WNDCLASSEXW, WS_VISIBLE,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class_name = wide_null("EditControlExample".as_ref());

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        // Fixed: Check return value instead of using ?
        let atom = RegisterClassExW(&wc);
        if atom == 0 {
            return Err(Error::from_thread());
        }

        let window_title = wide_null("Edit Control Readonly Example".as_ref());
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_title.as_ptr()),
            WINDOW_STYLE(0x00CF0000), // WS_OVERLAPPEDWINDOW
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        let edit_text = wide_null("This text is readonly".as_ref());
        let edit_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            WC_EDITW,
            PCWSTR(edit_text.as_ptr()),
            WINDOW_STYLE(WS_VISIBLE.0 | (ES_AUTOHSCROLL as u32) | (ES_READONLY as u32)),
            50,
            50,
            300,
            30,
            Some(hwnd),
            None,
            Some(instance.into()),
            None,
        )?;

        // Verify the readonly state by checking the window style
        let style = GetWindowLongW(edit_hwnd, GWL_STYLE);
        let is_readonly = (style as u32 & (ES_READONLY as u32)) != 0;
        println!("Edit control readonly state: {}", is_readonly);

        // Toggle readonly state off
        let result = SendMessageW(edit_hwnd, EM_SETREADONLY, Some(WPARAM(0)), Some(LPARAM(0)));
        if result.0 == 0 {
            return Err(Error::from_thread());
        }

        // Verify the readonly state was changed
        let style = GetWindowLongW(edit_hwnd, GWL_STYLE);
        let is_readonly = (style as u32 & (ES_READONLY as u32)) != 0;
        println!("Edit control readonly state after toggle: {}", is_readonly);

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).into() {
            TranslateMessage(&message);
            DispatchMessageW(&message);
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
