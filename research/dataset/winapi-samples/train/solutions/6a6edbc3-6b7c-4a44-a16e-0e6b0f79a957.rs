use windows::Win32::UI::WindowsAndMessaging::{GetDesktopWindow, IsWindowVisible};

fn main() {
    // Get the desktop window handle
    let desktop_hwnd = unsafe { GetDesktopWindow() };

    // Check if the desktop window is visible
    let is_visible = unsafe { IsWindowVisible(desktop_hwnd) };

    println!("Desktop window is visible: {}", is_visible.as_bool());
}
