// Vertically center text in a static label using non-client area adjustment

use std::mem;
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateSolidBrush, DeleteObject, DrawTextW, FillRect, GetDC, ReleaseDC, ScreenToClient,
    SelectObject, DT_CALCRECT, DT_LEFT, HBRUSH, HGDIOBJ,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW, GetWindowRect,
    GetWindowTextLengthW, GetWindowTextW, PostQuitMessage, RegisterClassW, SendMessageW,
    SetWindowPos, TranslateMessage, BS_LEFT, BS_NOTIFY, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, MSG,
    NCCALCSIZE_PARAMS, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY, WM_GETFONT, WM_NCCALCSIZE, WM_NCPAINT, WM_SIZE,
    WNDCLASSW, WS_CHILD, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

unsafe fn GetWindowFont(hwnd: HWND) -> HGDIOBJ {
    let result = SendMessageW(hwnd, WM_GETFONT, Some(WPARAM(0)), Some(LPARAM(0)));
    HGDIOBJ(result.0 as *mut _)
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe extern "system" fn label_subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _uidsubclass: usize,
    dwrefdata: usize,
) -> LRESULT {
    let brush = HBRUSH(dwrefdata as *mut _);

    match msg {
        WM_NCCALCSIZE => {
            if wparam.0 != 0 {
                // Calculate client area height needed for the font
                let font_handle = GetWindowFont(hwnd);
                let mut r: RECT = mem::zeroed();
                let dc = GetDC(Some(hwnd));

                let old = SelectObject(dc, font_handle);

                // Get window text
                let buffer_size = GetWindowTextLengthW(hwnd) as usize;
                let mut buffer = vec![0u16; buffer_size + 1];
                let text_len = GetWindowTextW(hwnd, &mut buffer);

                let mut newline_count = 1;
                if text_len > 0 {
                    for &c in buffer.iter().take(text_len as usize) {
                        if c == b'\n' as u16 {
                            newline_count += 1;
                        }
                    }
                    DrawTextW(
                        dc,
                        &mut buffer[..text_len as usize],
                        &mut r,
                        DT_CALCRECT | DT_LEFT,
                    );
                } else {
                    // Empty text - use a sample character for height calculation
                    let mut sample = wide_null("X");
                    DrawTextW(dc, &mut sample, &mut r, DT_CALCRECT | DT_LEFT);
                }

                let client_height = r.bottom * newline_count;

                SelectObject(dc, old);
                ReleaseDC(Some(hwnd), dc);

                // Calculate NC area to center text vertically
                let mut client: RECT = mem::zeroed();
                let mut window: RECT = mem::zeroed();
                let _ = GetClientRect(hwnd, &mut client);
                let _ = GetWindowRect(hwnd, &mut window);

                let window_height = window.bottom - window.top;
                let info_ptr: *mut NCCALCSIZE_PARAMS = lparam.0 as *mut NCCALCSIZE_PARAMS;
                let info = &mut *info_ptr;

                // Center the text vertically
                let center = ((window_height - client_height) / 2).max(0);
                info.rgrc[0].top += center;
                info.rgrc[0].bottom -= center;
            }
            LRESULT(0)
        }
        WM_NCPAINT => {
            // Paint the non-client area with the background brush
            let mut window: RECT = mem::zeroed();
            let mut client: RECT = mem::zeroed();
            let _ = GetWindowRect(hwnd, &mut window);
            let _ = GetClientRect(hwnd, &mut client);

            let mut pt1 = POINT {
                x: window.left,
                y: window.top,
            };
            let _ = ScreenToClient(hwnd, &mut pt1);

            let mut pt2 = POINT {
                x: window.right,
                y: window.bottom,
            };
            let _ = ScreenToClient(hwnd, &mut pt2);

            // Top non-client area
            let top = RECT {
                left: 0,
                top: pt1.y,
                right: client.right,
                bottom: client.top,
            };

            // Bottom non-client area
            let bottom = RECT {
                left: 0,
                top: client.bottom,
                right: client.right,
                bottom: pt2.y,
            };

            let dc = GetDC(Some(hwnd));
            let _ = FillRect(dc, &top, brush);
            let _ = FillRect(dc, &bottom, brush);
            let _ = ReleaseDC(Some(hwnd), dc);

            // Call default proc to handle the rest of NC painting
            DefSubclassProc(hwnd, msg, wparam, lparam)
        }
        WM_SIZE => {
            // Force recalculation of non-client area
            let _ = SetWindowPos(
                hwnd,
                Some(HWND::default()),
                0,
                0,
                0,
                0,
                SWP_NOOWNERZORDER | SWP_NOSIZE | SWP_NOMOVE | SWP_FRAMECHANGED,
            );
            DefSubclassProc(hwnd, msg, wparam, lparam)
        }
        _ => DefSubclassProc(hwnd, msg, wparam, lparam),
    }
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;

        // Register window class
        let class_name = wide_null("VerticalCenterLabelExample");
        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            hInstance: instance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        if RegisterClassW(&wc) == 0 {
            return Err(Error::from_thread());
        }

        // Create main window
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(wide_null("Vertical Center Label Example").as_ptr()),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            Some(HINSTANCE(instance.0)),
            None,
        )?;

        // Create a label with multiple lines
        let label_text = "Line 1\nLine 2\nLine 3";
        let label_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(wide_null("STATIC").as_ptr()),
            PCWSTR(wide_null(label_text).as_ptr()),
            WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | BS_LEFT as u32 | BS_NOTIFY as u32),
            50,
            50,
            300,
            200,
            Some(hwnd),
            None,
            Some(HINSTANCE(instance.0)),
            None,
        )?;

        // Create a white brush for the label background
        let brush = CreateSolidBrush(COLORREF(0x00FFFFFF)); // White

        // Subclass the label to handle vertical centering
        let result = SetWindowSubclass(label_hwnd, Some(label_subclass_proc), 0, brush.0 as usize);

        if !result.as_bool() {
            let _ = DeleteObject(brush.into());
            return Err(Error::from_thread());
        }

        // Message loop
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Clean up brush (subclass will handle deletion via WM_DESTROY if needed)
        // Note: In a real application, you'd want to handle cleanup more carefully

        Ok(())
    }
}
