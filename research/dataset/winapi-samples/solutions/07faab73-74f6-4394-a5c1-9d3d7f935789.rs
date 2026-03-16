// Set the font of a button control

use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateFontW, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_PITCH, DEFAULT_QUALITY, FF_DONTCARE,
    FW_BOLD, OUT_DEFAULT_PRECIS,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassExW, SendMessageW, ShowWindow, TranslateMessage, CW_USEDEFAULT, MSG, SW_SHOW,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY, WM_SETFONT, WNDCLASSEXW,
};

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;

        let class_name = wide_null("FontExampleWindowClass");
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        if RegisterClassExW(&wc) == 0 {
            return Err(Error::from_thread());
        }

        let window_title = wide_null("Font Example");
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

        let button_text = wide_null("Styled Button");
        let button_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(wide_null("BUTTON").as_ptr()),
            PCWSTR(button_text.as_ptr()),
            WINDOW_STYLE(0x50010000), // WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON
            50,
            50,
            200,
            50,
            Some(hwnd),
            None,
            Some(instance.into()),
            None,
        )?;

        // Create a custom font
        let font_name = wide_null("Arial");
        let hfont = CreateFontW(
            -24,              // Height
            0,                // Width (0 = auto)
            0,                // Escapement
            0,                // Orientation
            FW_BOLD.0 as i32, // Weight (FW_BOLD)
            0,                // Italic
            0,                // Underline
            0,                // StrikeOut
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            DEFAULT_QUALITY,
            (DEFAULT_PITCH.0 as u32) | (FF_DONTCARE.0 as u32), // Fixed: cast to u32 before OR
            PCWSTR(font_name.as_ptr()),
        );

        if hfont.0.is_null() {
            return Err(Error::from_thread());
        }

        // Set the font on the button
        // WM_SETFONT: wParam = font handle, lParam = redraw flag
        let _result = SendMessageW(
            button_hwnd,
            WM_SETFONT,
            Some(WPARAM(hfont.0 as usize)), // Fixed: wrapped in Some()
            Some(LPARAM(1)),                // Fixed: wrapped in Some()
        );

        // Show the window
        ShowWindow(hwnd, SW_SHOW);

        // Message loop
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
