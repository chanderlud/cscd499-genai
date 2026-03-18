// Get client rectangle of a window using ClientToScreen and GetClientRect

use windows::core::Result;
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::UI::WindowsAndMessaging::{GetClientRect, GetDesktopWindow};

fn get_client_rect(hwnd: HWND) -> Result<RECT> {
    let mut rect = RECT::default();
    let mut top_left = POINT { x: 0, y: 0 };

    unsafe {
        // Convert client coordinates (0,0) to screen coordinates
        ClientToScreen(hwnd, &mut top_left).ok()?;
        // Get the client area rectangle
        GetClientRect(hwnd, &mut rect)?;
    }

    // Adjust the rectangle by the screen coordinates of the top-left corner
    rect.left += top_left.x;
    rect.top += top_left.y;
    rect.right += top_left.x;
    rect.bottom += top_left.y;

    Ok(rect)
}

fn main() -> Result<()> {
    // Get the desktop window handle
    let desktop = unsafe { GetDesktopWindow() };

    // Get the client rectangle of the desktop window
    let client_rect = get_client_rect(desktop)?;

    println!("Desktop client rectangle:");
    println!("  Left: {}", client_rect.left);
    println!("  Top: {}", client_rect.top);
    println!("  Right: {}", client_rect.right);
    println!("  Bottom: {}", client_rect.bottom);

    Ok(())
}
