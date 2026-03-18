//! Demonstration of getting and setting console input modes using Windows API.

use windows::core::Result;
use windows::Win32::Foundation::{INVALID_HANDLE_VALUE, WAIT_FAILED, WAIT_OBJECT_0};
use windows::Win32::System::Console::{
    GetConsoleMode, GetStdHandle, SetConsoleMode, CONSOLE_MODE, ENABLE_MOUSE_INPUT,
    STD_INPUT_HANDLE,
};
use windows::Win32::System::Threading::{WaitForSingleObject, INFINITE};

fn main() -> Result<()> {
    // Step 1: Get the standard input handle
    let stdin_handle = unsafe { GetStdHandle(STD_INPUT_HANDLE) }?;

    if stdin_handle == INVALID_HANDLE_VALUE {
        return Err(std::io::Error::last_os_error().into());
    }

    println!("Successfully obtained stdin handle: {:?}", stdin_handle);

    // Step 2: Get current console mode for stdin
    let mut stdin_mode = CONSOLE_MODE::default();
    unsafe { GetConsoleMode(stdin_handle, &mut stdin_mode)?; }
    println!("Current stdin mode: {:?}", stdin_mode);

    // Step 3: Enable mouse input by OR-ing with ENABLE_MOUSE_INPUT
    let new_mode = stdin_mode | ENABLE_MOUSE_INPUT;

    // Step 4: Set the new console mode
    unsafe { SetConsoleMode(stdin_handle, new_mode)?; }
    println!("Successfully set stdin mode to: {:?}", new_mode);

    // Step 5: Verify the mode was set correctly by getting it again
    let mut verify_mode = CONSOLE_MODE::default();
    unsafe { GetConsoleMode(stdin_handle, &mut verify_mode)?; }
    println!("Verified stdin mode: {:?}", verify_mode);

    // Step 6: Demonstrate checking for pending input without blocking
    let wait_result = unsafe { WaitForSingleObject(stdin_handle, 0) };
    match wait_result {
        WAIT_OBJECT_0 => println!("Input is available (non-blocking check)"),
        WAIT_FAILED => return Err(std::io::Error::last_os_error().into()),
        _ => println!("No input available (blocking check)"),
    }

    // Step 7: Wait for actual input with INFINITE timeout
    println!("Waiting for input (press any key and press Enter)...");
    let wait_result = unsafe { WaitForSingleObject(stdin_handle, INFINITE) };
    match wait_result {
        WAIT_OBJECT_0 => println!("Input received!"),
        WAIT_FAILED => return Err(std::io::Error::last_os_error().into()),
        _ => return Err(std::io::Error::last_os_error().into()),
    }

    Ok(())
}
