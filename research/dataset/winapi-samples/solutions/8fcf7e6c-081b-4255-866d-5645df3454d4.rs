// Send EM_UNDO message to an edit control

use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::EM_UNDO;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassW, SendMessageW, TranslateMessage, CW_USEDEFAULT, ES_AUTOHSCROLL, MSG,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_CREATE, WM_DESTROY, WNDCLASSW, WS_BORDER, WS_CHILD,
    WS_OVERLAPPEDWINDOW, WS_TABSTOP, WS_VISIBLE,
};

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

fn main() -> Result<()> {
    let instance = unsafe { GetModuleHandleW(None)? };
    let class_name = wide_null("UndoExampleClass");

    let wc = WNDCLASSW {
        lpfnWndProc: Some(wndproc),
        hInstance: instance.into(),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        ..Default::default()
    };

    if unsafe { RegisterClassW(&wc) } == 0 {
        return Err(Error::from_thread());
    }

    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(wide_null("EM_UNDO Example").as_ptr()),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            Some(instance.into()),
            None,
        )
    }?;

    if hwnd.is_invalid() {
        return Err(Error::from_thread());
    }

    let mut msg = MSG::default();
    while unsafe { GetMessageW(&mut msg, None, 0, 0) }.into() {
        unsafe {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    Ok(())
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            let edit_class = wide_null("EDIT");
            let edit_text = wide_null("Initial text - try typing then undo");

            let edit_hwnd = unsafe {
                CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    PCWSTR(edit_class.as_ptr()),
                    PCWSTR(edit_text.as_ptr()),
                    WS_CHILD
                        | WS_VISIBLE
                        | WS_BORDER
                        | WINDOW_STYLE(ES_AUTOHSCROLL as u32)
                        | WS_TABSTOP,
                    50,
                    50,
                    300,
                    25,
                    Some(hwnd),
                    None,
                    None,
                    None,
                )
            };

            if let Ok(edit_hwnd) = edit_hwnd {
                if !edit_hwnd.is_invalid() {
                    // Demonstrate EM_UNDO by programmatically sending it
                    unsafe { SendMessageW(edit_hwnd, EM_UNDO, Some(WPARAM(0)), Some(LPARAM(0))) };
                }
            }

            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
