use windows::core::Result;
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::WindowsAndMessaging::{ClipCursor, GetClipCursor};

fn get_cursor_clip() -> Result<RECT> {
    unsafe {
        let mut rect = RECT::default();
        GetClipCursor(&mut rect)?;
        Ok(rect)
    }
}

fn set_cursor_clip(rect: Option<RECT>) -> Result<()> {
    unsafe {
        let rect_ptr = rect.as_ref().map(|r| r as *const RECT);
        ClipCursor(rect_ptr)
    }
}

fn main() -> Result<()> {
    // Get current cursor clip rectangle
    let original_clip = get_cursor_clip()?;
    println!("Original cursor clip: {:?}", original_clip);

    // Set a new clip rectangle (100x100 at position 200,200)
    let new_clip = RECT {
        left: 200,
        top: 200,
        right: 300,
        bottom: 300,
    };
    set_cursor_clip(Some(new_clip))?;
    println!("Set new cursor clip: {:?}", new_clip);

    // Verify the change
    let current_clip = get_cursor_clip()?;
    println!("Current cursor clip: {:?}", current_clip);

    // Restore original clip
    set_cursor_clip(Some(original_clip))?;
    println!("Restored original cursor clip");

    Ok(())
}
