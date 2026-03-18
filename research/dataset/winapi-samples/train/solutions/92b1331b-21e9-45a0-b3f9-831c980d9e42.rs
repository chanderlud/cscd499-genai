use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use windows::Win32::UI::WindowsAndMessaging::ShowCursor;

fn set_cursor_hidden(hidden: bool) {
    static HIDDEN: AtomicBool = AtomicBool::new(false);
    let changed = HIDDEN.swap(hidden, Ordering::SeqCst) ^ hidden;
    if changed {
        // SAFETY: ShowCursor is safe to call with a boolean.
        unsafe { ShowCursor(!hidden) };
    }
}

fn main() {
    // Hide the cursor
    set_cursor_hidden(true);

    // Wait for 3 seconds
    thread::sleep(Duration::from_secs(3));

    // Show the cursor again
    set_cursor_hidden(false);
}
