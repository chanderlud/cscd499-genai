use windows::Win32::Foundation::RECT;
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};

fn get_desktop_rect() -> RECT {
    unsafe {
        let left = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let top = GetSystemMetrics(SM_YVIRTUALSCREEN);
        RECT {
            left,
            top,
            right: left + GetSystemMetrics(SM_CXVIRTUALSCREEN),
            bottom: top + GetSystemMetrics(SM_CYVIRTUALSCREEN),
        }
    }
}

fn main() {
    let rect = get_desktop_rect();
    println!("Virtual screen rectangle:");
    println!("  Left: {}", rect.left);
    println!("  Top: {}", rect.top);
    println!("  Right: {}", rect.right);
    println!("  Bottom: {}", rect.bottom);
    println!("  Width: {}", rect.right - rect.left);
    println!("  Height: {}", rect.bottom - rect.top);
}
