// TITLE: Retrieve button image type and handle using BM_GETIMAGE

use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{CreateSolidBrush, GetSysColor, COLOR_WINDOW};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, LoadIconW, MessageBoxW,
    PostQuitMessage, RegisterClassW, SendMessageW, TranslateMessage, BM_GETIMAGE, BM_SETIMAGE,
    BS_ICON, IDI_APPLICATION, IMAGE_BITMAP, IMAGE_ICON, MB_OK, MSG, WINDOW_EX_STYLE, WINDOW_STYLE,
    WM_CREATE, WM_DESTROY, WNDCLASSW,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class_name = wide_null("ButtonImageExample".as_ref());

        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: HINSTANCE(instance.0),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            hbrBackground: CreateSolidBrush(COLORREF(GetSysColor(COLOR_WINDOW))),
            ..Default::default()
        };

        let atom = RegisterClassW(&wc);
        if atom == 0 {
            return Err(windows::core::Error::from_thread());
        }

        let window_title = wide_null("Button Image Retrieval Example".as_ref());
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_title.as_ptr()),
            WINDOW_STYLE(0x00CF0000), // WS_OVERLAPPEDWINDOW
            100,
            100,
            400,
            300,
            None,
            None,
            Some(HINSTANCE(instance.0)),
            None,
        )?;

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Ok(())
    }
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => {
                // Create a button with an icon
                let button_text = wide_null("Icon Button".as_ref());
                let button_hwnd = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    PCWSTR(wide_null("BUTTON".as_ref()).as_ptr()),
                    PCWSTR(button_text.as_ptr()),
                    WINDOW_STYLE(BS_ICON as u32 | 0x50010000), // BS_ICON | WS_VISIBLE | WS_CHILD | WS_TABSTOP
                    50,
                    50,
                    120,
                    30,
                    Some(hwnd),
                    None,
                    None,
                    None,
                )
                .unwrap();

                // Load and set an icon
                let icon = LoadIconW(None, IDI_APPLICATION).unwrap();
                let _ = SendMessageW(
                    button_hwnd,
                    BM_SETIMAGE,
                    Some(WPARAM(IMAGE_ICON.0 as usize)),
                    Some(LPARAM(icon.0 as isize)),
                );

                // Retrieve the image to demonstrate BM_GETIMAGE
                let bitmap_result = SendMessageW(
                    button_hwnd,
                    BM_GETIMAGE,
                    Some(WPARAM(IMAGE_BITMAP.0 as usize)),
                    Some(LPARAM(0)),
                );

                let icon_result = SendMessageW(
                    button_hwnd,
                    BM_GETIMAGE,
                    Some(WPARAM(IMAGE_ICON.0 as usize)),
                    Some(LPARAM(0)),
                );

                // Check what type of image the button has
                if bitmap_result.0 != 0 {
                    let _ = MessageBoxW(
                        Some(hwnd),
                        PCWSTR(wide_null("Button has a bitmap image".as_ref()).as_ptr()),
                        PCWSTR(wide_null("Image Type".as_ref()).as_ptr()),
                        MB_OK,
                    );
                } else if icon_result.0 != 0 {
                    let _ = MessageBoxW(
                        Some(hwnd),
                        PCWSTR(wide_null("Button has an icon image".as_ref()).as_ptr()),
                        PCWSTR(wide_null("Image Type".as_ref()).as_ptr()),
                        MB_OK,
                    );
                } else {
                    let _ = MessageBoxW(
                        Some(hwnd),
                        PCWSTR(wide_null("Button has no image".as_ref()).as_ptr()),
                        PCWSTR(wide_null("Image Type".as_ref()).as_ptr()),
                        MB_OK,
                    );
                }

                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
