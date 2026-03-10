//! Demonstrates restoring original console modes on Windows.
//!
//! This program:
//! 1. Retrieves standard input, output, and error handles
//! 2. Queries the current console mode flags for each handle
//! 3. Modifies the console modes (sets raw mode for stdin)
//! 4. Restores the original console modes
//! 5. Checks if stdin has pending input without blocking

use windows::core::Result;
use windows::Win32::Foundation::{HANDLE, WAIT_OBJECT_0};
use windows::Win32::System::Console::{
    GetConsoleMode, GetStdHandle, SetConsoleMode, CONSOLE_MODE, ENABLE_ECHO_INPUT,
    ENABLE_EXTENDED_FLAGS, ENABLE_INSERT_MODE, ENABLE_LINE_INPUT, ENABLE_MOUSE_INPUT,
    ENABLE_PROCESSED_INPUT, ENABLE_QUICK_EDIT_MODE, ENABLE_VIRTUAL_TERMINAL_INPUT,
    STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
};
use windows::Win32::System::Threading::WaitForSingleObject;

/// Represents a console session with opened handles and their original modes.
struct ConsoleSession {
    stdin: HANDLE,
    stdout: HANDLE,
    stderr: HANDLE,
    stdin_mode: CONSOLE_MODE,
    stdout_mode: CONSOLE_MODE,
    stderr_mode: CONSOLE_MODE,
}

impl ConsoleSession {
    /// Creates a new console session from default stdin, stdout, and stderr handles.
    fn current() -> Result<Self> {
        // Retrieve standard handles. These are not closed on drop as they are copies
        // of values stored in the process table.
        let stdin = unsafe { GetStdHandle(STD_INPUT_HANDLE)? };
        let stdout = unsafe { GetStdHandle(STD_OUTPUT_HANDLE)? };
        let stderr = unsafe { GetStdHandle(STD_ERROR_HANDLE)? };

        // Query the current console mode for each handle.
        let stdin_mode = get_console_mode(stdin)?;
        let stdout_mode = get_console_mode(stdout)?;
        let stderr_mode = get_console_mode(stderr)?;

        Ok(Self {
            stdin,
            stdout,
            stderr,
            stdin_mode,
            stdout_mode,
            stderr_mode,
        })
    }

    /// Sets stdin to raw mode by clearing input processing flags and setting
    /// extended flags for raw terminal behavior.
    fn set_raw_stdin(&self) -> Result<()> {
        let mut mode = self.stdin_mode;

        // Clear input processing flags to disable echo, line input, and mouse input.
        mode &= !ENABLE_ECHO_INPUT;
        mode &= !ENABLE_LINE_INPUT;
        mode &= !ENABLE_MOUSE_INPUT;
        mode &= !ENABLE_PROCESSED_INPUT;

        // Set extended flags for raw terminal behavior.
        mode |= ENABLE_EXTENDED_FLAGS;
        mode |= ENABLE_INSERT_MODE;
        mode |= ENABLE_QUICK_EDIT_MODE;

        // Enable virtual terminal input for ANSI escape sequence support.
        mode |= ENABLE_VIRTUAL_TERMINAL_INPUT;

        unsafe { SetConsoleMode(self.stdin, mode)? };
        Ok(())
    }

    /// Restores the original console modes for all handles.
    fn restore_modes(&self) -> Result<()> {
        // Restore stdin mode.
        unsafe { SetConsoleMode(self.stdin, self.stdin_mode)? };

        // Restore stdout mode.
        unsafe { SetConsoleMode(self.stdout, self.stdout_mode)? };

        // Restore stderr mode.
        unsafe { SetConsoleMode(self.stderr, self.stderr_mode)? };

        Ok(())
    }

    /// Checks if stdin has pending input without blocking.
    fn is_stdin_empty(&self) -> Result<bool> {
        // Wait for input with zero timeout. If input is available, WaitForSingleObject
        // returns WAIT_OBJECT_0 immediately.
        let result = unsafe { WaitForSingleObject(self.stdin, 0) };
        Ok(result == WAIT_OBJECT_0)
    }
}

/// Queries the current console mode for the given handle.
fn get_console_mode(handle: HANDLE) -> Result<CONSOLE_MODE> {
    let mut mode = CONSOLE_MODE::default();
    unsafe { GetConsoleMode(handle, &mut mode)? };
    Ok(mode)
}

fn main() -> Result<()> {
    println!("=== Console Mode Restoration Demo ===\n");

    // Create a console session and capture original modes.
    let console = ConsoleSession::current()?;
    println!("✓ Captured original console modes");
    println!("  stdin mode: 0x{:08X}", console.stdin_mode.0);
    println!("  stdout mode: 0x{:08X}", console.stdout_mode.0);
    println!("  stderr mode: 0x{:08X}\n", console.stderr_mode.0);

    // Set stdin to raw mode for raw terminal behavior.
    console.set_raw_stdin()?;
    println!("✓ Set stdin to raw mode");
    println!("  (echo, line input, and mouse input disabled)\n");

    // Check if stdin has pending input.
    let empty = console.is_stdin_empty()?;
    println!("✓ Stdin empty: {}", empty);
    println!("  (input available: {})\n", !empty);

    // Restore original console modes.
    console.restore_modes()?;
    println!("✓ Restored original console modes");
    println!("  stdin mode: 0x{:08X}", console.stdin_mode.0);
    println!("  stdout mode: 0x{:08X}", console.stdout_mode.0);
    println!("  stderr mode: 0x{:08X}\n", console.stderr_mode.0);

    println!("=== Demo Complete ===");
    Ok(())
}
