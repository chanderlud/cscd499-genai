//! Demonstrates checking stdin for available input on Windows using Win32 API.

use windows::core::Result as WinResult;
use windows::Win32::Foundation::{INVALID_HANDLE_VALUE, WAIT_OBJECT_0};
use windows::Win32::System::Console::{
    GetConsoleMode, GetStdHandle, SetConsoleMode, CONSOLE_MODE, ENABLE_PROCESSED_INPUT,
    ENABLE_VIRTUAL_TERMINAL_INPUT, STD_INPUT_HANDLE,
};
use windows::Win32::System::Threading::WaitForSingleObject;

/// Checks if there is pending input available on stdin without blocking.
///
/// This function uses WaitForSingleObject with a zero timeout to determine
/// if the stdin handle is signaled, which indicates that input is available.
///
/// Returns:
/// - `Ok(true)` if input is available
/// - `Ok(false)` if no input is available
/// - `Err` if there's an error getting the stdin handle or console mode
fn is_stdin_available() -> WinResult<bool> {
    // Get the standard input handle
    let stdin_handle = unsafe { GetStdHandle(STD_INPUT_HANDLE)? };

    // Check if the handle is valid (INVALID_HANDLE_VALUE is not an error but indicates failure)
    if stdin_handle == INVALID_HANDLE_VALUE {
        return Err(windows::core::Error::from_thread());
    }

    // Query current console mode to ensure we can set it properly
    let mut current_mode = CONSOLE_MODE::default();
    unsafe { GetConsoleMode(stdin_handle, &mut current_mode)? };

    // Set console mode to enable processed input and virtual terminal input
    // This is necessary for WaitForSingleObject to work correctly
    let desired_mode = ENABLE_PROCESSED_INPUT | ENABLE_VIRTUAL_TERMINAL_INPUT;
    unsafe { SetConsoleMode(stdin_handle, desired_mode)? };

    // Check if input is available without blocking
    // WAIT_OBJECT_0 indicates the handle is signaled (input available)
    let available = unsafe { WaitForSingleObject(stdin_handle, 0) == WAIT_OBJECT_0 };

    // Restore the original console mode
    unsafe { SetConsoleMode(stdin_handle, current_mode)? };

    Ok(available)
}

fn main() -> WinResult<()> {
    println!("Checking stdin for available input...");

    match is_stdin_available() {
        Ok(true) => {
            println!("✓ Input is available on stdin");
            println!("  You can safely call std::io::stdin().read() without blocking");
        }
        Ok(false) => {
            println!("✗ No input available on stdin");
            println!("  The next read() call would block");
        }
        Err(e) => {
            eprintln!("Error checking stdin: {}", e);
            return Err(e);
        }
    }

    // Demonstrate with a simple read attempt
    println!("\nAttempting to read a line from stdin...");
    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(0) => println!("EOF reached (no more input)"),
        Ok(n) => println!("Read {} bytes: '{}'", n, input.trim()),
        Err(e) => eprintln!("Read error: {}", e),
    }

    Ok(())
}
