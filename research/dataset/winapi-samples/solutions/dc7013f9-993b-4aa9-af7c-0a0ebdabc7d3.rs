// TITLE: Adjust window rectangle for non-client area with DPI awareness

use std::sync::OnceLock;
use windows::{
    core::{Error, Result, PCWSTR},
    Win32::{
        Foundation::{HWND, RECT},
        UI::WindowsAndMessaging::{
            AdjustWindowRectEx, GetMenu, GetWindowLongW, GWL_EXSTYLE, GWL_STYLE, WINDOW_EX_STYLE,
            WINDOW_STYLE,
        },
    },
};

// Helper function to convert string to wide null-terminated string
fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

// Helper function to dynamically load function pointer
fn get_function_impl(
    library: &str,
    function: &str,
) -> Option<unsafe extern "system" fn() -> isize> {
    use windows::core::PCSTR;
    use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

    let library_wide = wide_null(std::ffi::OsStr::new(library));
    let module = unsafe { LoadLibraryW(PCWSTR::from_raw(library_wide.as_ptr())) }.ok()?;
    if module.is_invalid() {
        return None;
    }

    let function_bytes = function.as_bytes();
    let function_ptr = unsafe { GetProcAddress(module, PCSTR::from_raw(function_bytes.as_ptr())) };
    function_ptr.map(|f| unsafe { std::mem::transmute(f) })
}

// Type for AdjustWindowRectExForDpi function
type AdjustWindowRectExForDpi = unsafe extern "system" fn(
    rect: *mut RECT,
    dwStyle: WINDOW_STYLE,
    bMenu: bool,
    dwExStyle: WINDOW_EX_STYLE,
    dpi: u32,
) -> i32;

// Static to hold the dynamically loaded function
static ADJUST_WINDOW_RECT_EX_FOR_DPI: OnceLock<Option<AdjustWindowRectExForDpi>> = OnceLock::new();

// Initialize the function pointer
fn init_adjust_window_rect_ex_for_dpi() -> Option<AdjustWindowRectExForDpi> {
    get_function_impl("user32.dll", "AdjustWindowRectExForDpi\0")
        .map(|f| unsafe { std::mem::transmute(f) })
}

// Adjust window rectangle considering DPI scaling
pub fn adjust_window_rect_with_dpi(
    hwnd: HWND,
    mut rect: RECT,
    style: WINDOW_STYLE,
    style_ex: WINDOW_EX_STYLE,
    dpi: u32,
) -> Result<RECT> {
    let b_menu = !unsafe { GetMenu(hwnd) }.is_invalid();

    // Try to use DPI-aware version if available
    if let Some(adjust_func) =
        ADJUST_WINDOW_RECT_EX_FOR_DPI.get_or_init(init_adjust_window_rect_ex_for_dpi)
    {
        let success = unsafe { adjust_func(&mut rect, style, b_menu, style_ex, dpi) };
        if success != 0 {
            return Ok(rect);
        }
    }

    // Fall back to non-DPI-aware version
    unsafe { AdjustWindowRectEx(&mut rect, style, b_menu, style_ex)? };
    Ok(rect)
}

// Adjust window rectangle based on window's current style
pub fn adjust_window_rect(hwnd: HWND, rect: RECT, is_decorated: bool) -> Result<RECT> {
    unsafe {
        let mut style = WINDOW_STYLE(GetWindowLongW(hwnd, GWL_STYLE) as u32);
        // If window isn't decorated, remove caption and sizebox styles
        if !is_decorated {
            style &= !WINDOW_STYLE(0x00C00000); // WS_CAPTION
            style &= !WINDOW_STYLE(0x00040000); // WS_SIZEBOX
        }
        let style_ex = WINDOW_EX_STYLE(GetWindowLongW(hwnd, GWL_EXSTYLE) as u32);

        // For simplicity, use 96 DPI (standard) in this example
        // In real code, you'd get the actual DPI for the window
        adjust_window_rect_with_dpi(hwnd, rect, style, style_ex, 96)
    }
}

fn main() -> Result<()> {
    // Get desktop window as an example
    let desktop_hwnd = unsafe { windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow() };

    // Define a client area rectangle
    let client_rect = RECT {
        left: 100,
        top: 100,
        right: 900,  // 800 pixels wide
        bottom: 700, // 600 pixels tall
    };

    println!("Original client rectangle: {:?}", client_rect);

    // Adjust for decorated window (with title bar, borders, etc.)
    let decorated_rect = adjust_window_rect(desktop_hwnd, client_rect, true)?;
    println!("Decorated window rectangle: {:?}", decorated_rect);

    // Adjust for undecorated window (no title bar or borders)
    let undecorated_rect = adjust_window_rect(desktop_hwnd, client_rect, false)?;
    println!("Undecorated window rectangle: {:?}", undecorated_rect);

    // Calculate the difference (non-client area)
    let nc_width_decorated =
        (decorated_rect.right - decorated_rect.left) - (client_rect.right - client_rect.left);
    let nc_height_decorated =
        (decorated_rect.bottom - decorated_rect.top) - (client_rect.bottom - client_rect.top);

    println!("\nNon-client area for decorated window:");
    println!("  Width: {} pixels", nc_width_decorated);
    println!("  Height: {} pixels", nc_height_decorated);

    Ok(())
}
