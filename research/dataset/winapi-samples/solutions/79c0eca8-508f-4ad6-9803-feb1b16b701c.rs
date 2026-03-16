// Set and retrieve font in an edit control

use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateFontW, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_PITCH, DEFAULT_QUALITY, FW_NORMAL,
    OUT_DEFAULT_PRECIS,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassExW, SendMessageW, TranslateMessage, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT,
    ES_AUTOHSCROLL, MSG, WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY, WM_GETFONT, WM_SETFONT,
    WNDCLASSEXW, WS_BORDER, WS_CHILD, WS_OVERLAPPEDWINDOW, WS_TABSTOP, WS_VISIBLE,
};

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;

        let class_name = wide_null("FontExampleClass");
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        let atom = RegisterClassExW(&wc);
        if atom == 0 {
            return Err(Error::from_thread());
        }

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(wide_null("Font Example").as_ptr()),
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

        let edit_class = wide_null("EDIT");
        let edit_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(edit_class.as_ptr()),
            PCWSTR(wide_null("Sample text").as_ptr()),
            WS_CHILD | WS_VISIBLE | WS_BORDER | WINDOW_STYLE(ES_AUTOHSCROLL as u32) | WS_TABSTOP,
            50,
            50,
            200,
            25,
            Some(hwnd),
            None,
            Some(instance.into()),
            None,
        )?;

        // Create a font
        let font_name = wide_null("Arial");
        let hfont = CreateFontW(
            -16,
            0,
            0,
            0,
            FW_NORMAL.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            DEFAULT_QUALITY,
            DEFAULT_PITCH.0 as u32,
            PCWSTR(font_name.as_ptr()),
        );

        if hfont.is_invalid() {
            return Err(Error::from_thread());
        }

        // Set the font to the edit control
        let _ = SendMessageW(
            edit_hwnd,
            WM_SETFONT,
            Some(WPARAM(hfont.0 as usize)),
            Some(LPARAM(1)), // TRUE - redraw
        );

        // Get the font from the edit control
        let retrieved_font = SendMessageW(edit_hwnd, WM_GETFONT, Some(WPARAM(0)), Some(LPARAM(0)));

        if retrieved_font.0 == 0 {
            println!("No font set on edit control");
        } else {
            println!("Font successfully set and retrieved from edit control");
        }

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            let _ = DispatchMessageW(&msg);
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
