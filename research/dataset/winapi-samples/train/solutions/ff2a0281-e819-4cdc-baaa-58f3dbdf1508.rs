use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND};
use windows::Win32::Graphics::Dwm::DwmExtendFrameIntoClientArea;
use windows::Win32::Graphics::Gdi::{InvalidateRect, UpdateWindow};
use windows::Win32::UI::Controls::MARGINS;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetWindowLongW, PostQuitMessage, RegisterClassExW,
    SetLayeredWindowAttributes, ShowWindow, CW_USEDEFAULT, GWL_EXSTYLE, LWA_ALPHA, LWA_COLORKEY,
    SW_SHOW, WINDOW_EX_STYLE, WM_DESTROY, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP, WS_VISIBLE,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn create_transparent_overlay(class_name: &str, width: i32, height: i32) -> Result<HWND> {
    let class_name_wide = wide_null(class_name.as_ref());

    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        lpfnWndProc: Some(wndproc),
        hInstance: unsafe { windows::Win32::System::LibraryLoader::GetModuleHandleW(None)?.into() },
        lpszClassName: PCWSTR(class_name_wide.as_ptr()),
        ..Default::default()
    };

    let atom = unsafe { RegisterClassExW(&wc) };
    if atom == 0 {
        return Err(Error::from_thread());
    }

    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(
                WS_EX_LAYERED.0
                    | WS_EX_TRANSPARENT.0
                    | WS_EX_TOPMOST.0
                    | WS_EX_NOACTIVATE.0
                    | WS_EX_TOOLWINDOW.0,
            ),
            PCWSTR(class_name_wide.as_ptr()),
            PCWSTR::null(),
            WS_POPUP | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            width,
            height,
            None,
            None,
            Some(wc.hInstance),
            None,
        )?
    };

    // Set initial color-key transparency (magenta)
    unsafe {
        SetLayeredWindowAttributes(hwnd, COLORREF(0x00FF00FF), 255, LWA_COLORKEY)?;
    }

    // Extend glass effect for proper transparency
    let margins = MARGINS {
        cxLeftWidth: -1,
        cxRightWidth: -1,
        cyTopHeight: -1,
        cyBottomHeight: -1,
    };
    unsafe {
        DwmExtendFrameIntoClientArea(hwnd, &margins)?;
    }

    unsafe {
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
    }

    Ok(hwnd)
}

pub fn toggle_transparency_mode(hwnd: HWND, use_alpha: bool, alpha_value: u8) -> Result<()> {
    unsafe {
        let _ex_style = WINDOW_EX_STYLE(GetWindowLongW(hwnd, GWL_EXSTYLE) as u32);

        if use_alpha {
            // Switch to alpha transparency
            SetLayeredWindowAttributes(hwnd, COLORREF(0), alpha_value, LWA_ALPHA)?;
        } else {
            // Switch to color-key transparency
            SetLayeredWindowAttributes(hwnd, COLORREF(0x00FF00FF), 255, LWA_COLORKEY)?;
        }

        // Force repaint to apply changes without flickering
        InvalidateRect(Some(hwnd), None, false);
        UpdateWindow(hwnd);
    }

    Ok(())
}

extern "system" fn wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    unsafe {
        match msg {
            WM_DESTROY => {
                PostQuitMessage(0);
                windows::Win32::Foundation::LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
