// Get window style and extended style using GetWindowLongW

use windows::core::Result;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongW, GWL_EXSTYLE, GWL_STYLE, WINDOW_EX_STYLE, WINDOW_STYLE,
};

fn get_window_styles(hwnd: HWND) -> Result<(WINDOW_STYLE, WINDOW_EX_STYLE)> {
    // SAFETY: GetWindowLongW is safe to call with a valid HWND and index
    let style = unsafe { GetWindowLongW(hwnd, GWL_STYLE) };
    let style_ex = unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) };

    Ok((WINDOW_STYLE(style as u32), WINDOW_EX_STYLE(style_ex as u32)))
}

fn main() -> Result<()> {
    // Get the desktop window as an example
    let desktop = unsafe { windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow() };

    let (style, style_ex) = get_window_styles(desktop)?;

    println!("Window style: 0x{:08X}", style.0);
    println!("Extended style: 0x{:08X}", style_ex.0);

    Ok(())
}
