use std::ffi::OsStr;
use std::iter::once;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::{EM_GETCUEBANNER, EM_SETCUEBANNER};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassExW, SendMessageW, TranslateMessage, CS_HREDRAW, CS_VREDRAW, ES_AUTOHSCROLL,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY, WNDCLASSEXW, WS_BORDER, WS_CHILD,
    WS_OVERLAPPEDWINDOW, WS_TABSTOP, WS_VISIBLE,
};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class_name = wide_null(OsStr::new("PlaceholderExampleClass"));

        let wc = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            hCursor: Default::default(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        if RegisterClassExW(&wc) == 0 {
            return Err(Error::from_thread());
        }

        let window_title = wide_null(OsStr::new("Placeholder Text Example"));
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_title.as_ptr()),
            WS_VISIBLE | WS_OVERLAPPEDWINDOW,
            100,
            100,
            400,
            300,
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        let edit_class = wide_null(OsStr::new("EDIT"));
        let edit_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(edit_class.as_ptr()),
            PCWSTR(ptr::null()),
            WS_VISIBLE | WS_CHILD | WS_BORDER | WS_TABSTOP | WINDOW_STYLE(ES_AUTOHSCROLL as u32),
            50,
            50,
            300,
            25,
            Some(hwnd),
            None,
            Some(instance.into()),
            None,
        )?;

        // Set placeholder text
        let placeholder = wide_null(OsStr::new("Enter text here..."));
        let result = SendMessageW(
            edit_hwnd,
            EM_SETCUEBANNER,
            Some(WPARAM(0)),
            Some(LPARAM(placeholder.as_ptr() as isize)),
        );

        if result.0 == 0 {
            return Err(Error::from_thread());
        }

        // Get placeholder text
        let mut buffer = vec![0u16; 100];
        let result = SendMessageW(
            edit_hwnd,
            EM_GETCUEBANNER,
            Some(WPARAM(buffer.as_mut_ptr() as usize)),
            Some(LPARAM(buffer.len() as isize)),
        );

        if result.0 != 0 {
            let retrieved = String::from_utf16_lossy(&buffer);
            println!(
                "Retrieved placeholder: {}",
                retrieved.trim_end_matches('\0')
            );
        }

        let mut message = Default::default();
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
