// TITLE: Setting Window Dark Mode with DwmSetWindowAttribute

use windows::core::{Result, BOOL};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE};
use windows::Win32::UI::Input::KeyboardAndMouse::GetActiveWindow;
use windows::Win32::UI::WindowsAndMessaging::{DefWindowProcW, WM_NCACTIVATE};

fn set_window_dark_mode(hwnd: HWND, is_dark_mode: bool, redraw_title_bar: bool) -> Result<()> {
    // Use the documented DWMWA_USE_IMMERSIVE_DARK_MODE attribute
    let dark_mode = BOOL::from(is_dark_mode);

    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &dark_mode as *const BOOL as *const _,
            std::mem::size_of::<BOOL>() as u32,
        )?;
    }

    if redraw_title_bar {
        unsafe {
            // Redraw title bar by toggling NC activation state
            if GetActiveWindow() == hwnd {
                DefWindowProcW(hwnd, WM_NCACTIVATE, WPARAM::default(), LPARAM::default());
                DefWindowProcW(hwnd, WM_NCACTIVATE, WPARAM(true.into()), LPARAM::default());
            } else {
                DefWindowProcW(hwnd, WM_NCACTIVATE, WPARAM(true.into()), LPARAM::default());
                DefWindowProcW(hwnd, WM_NCACTIVATE, WPARAM::default(), LPARAM::default());
            }
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    // Get the active window handle
    let hwnd = unsafe { GetActiveWindow() };

    if hwnd.0 != std::ptr::null_mut() {
        // Set dark mode for the window
        set_window_dark_mode(hwnd, true, true)?;
        println!("Dark mode enabled for the active window.");
    } else {
        println!("No active window found.");
    }

    Ok(())
}
