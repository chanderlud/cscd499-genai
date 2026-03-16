// TITLE: Create an edit control with center text alignment

use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{GetSysColorBrush, COLOR_WINDOW, HBRUSH};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassExW, TranslateMessage, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, ES_AUTOHSCROLL,
    ES_CENTER, MSG, WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY, WNDCLASSEXW, WS_BORDER, WS_CHILD,
    WS_OVERLAPPEDWINDOW, WS_TABSTOP, WS_VISIBLE,
};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class_name = wide_null(OsStr::new("CenterEditExample"));

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            hCursor: Default::default(),
            hbrBackground: GetSysColorBrush(COLOR_WINDOW),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        let atom = RegisterClassExW(&wc);
        if atom == 0 {
            return Err(Error::from_thread());
        }

        let window_title = wide_null(OsStr::new("Center Text Alignment Example"));
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

        // Create edit control with center alignment
        let edit_class = wide_null(OsStr::new("EDIT"));
        let edit_text = wide_null(OsStr::new("Centered Text"));

        let edit_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(edit_class.as_ptr()),
            PCWSTR(edit_text.as_ptr()),
            WS_CHILD
                | WS_VISIBLE
                | WS_BORDER
                | WS_TABSTOP
                | WINDOW_STYLE(ES_AUTOHSCROLL as u32)
                | WINDOW_STYLE(ES_CENTER as u32),
            50,
            50,
            300,
            30,
            Some(hwnd),
            None,
            Some(instance.into()),
            None,
        )?;

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
