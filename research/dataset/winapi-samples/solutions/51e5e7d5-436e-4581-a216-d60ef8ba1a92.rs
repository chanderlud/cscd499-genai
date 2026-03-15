use windows::core::Result;
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

fn main() -> Result<()> {
    let mut point = POINT::default();
    // SAFETY: GetCursorPos writes to a valid POINT struct
    unsafe { GetCursorPos(&mut point)? };

    println!("Cursor position: ({}, {})", point.x, point.y);
    Ok(())
}
