use windows::core::{Error, Result as WinResult};
use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE, WAIT_OBJECT_0};
use windows::Win32::System::Console::{
    GetConsoleMode, GetStdHandle, SetConsoleMode, CONSOLE_MODE, ENABLE_ECHO_INPUT,
    ENABLE_EXTENDED_FLAGS, ENABLE_INSERT_MODE, ENABLE_LINE_INPUT, ENABLE_MOUSE_INPUT,
    ENABLE_PROCESSED_INPUT, ENABLE_QUICK_EDIT_MODE, ENABLE_VIRTUAL_TERMINAL_INPUT,
    STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
};
use windows::Win32::System::Threading::WaitForSingleObject;

/// Represents a console session with opened standard handles.
#[derive(Debug, Clone)]
struct ConsoleSession {
    stdin: HANDLE,
    stdout: HANDLE,
    stderr: HANDLE,
}

impl ConsoleSession {
    /// Creates a new ConsoleSession from the default stdin, stdout, and stderr handles.
    ///
    /// Returns an error if any handle retrieval fails.
    fn new() -> WinResult<Self> {
        // Retrieve standard handles
        let stdin = unsafe { GetStdHandle(STD_INPUT_HANDLE)? };
        let stdout = unsafe { GetStdHandle(STD_OUTPUT_HANDLE)? };
        let stderr = unsafe { GetStdHandle(STD_ERROR_HANDLE)? };

        // Validate handles - GetStdHandle can return INVALID_HANDLE_VALUE on failure
        Self::validate_handle(stdin, "stdin")?;
        Self::validate_handle(stdout, "stdout")?;
        Self::validate_handle(stderr, "stderr")?;

        Ok(Self {
            stdin,
            stdout,
            stderr,
        })
    }

    /// Validates that a handle is not INVALID_HANDLE_VALUE.
    ///
    /// Returns an error with an HRESULT if the handle is invalid.
    fn validate_handle(handle: HANDLE, name: &str) -> WinResult<()> {
        if handle == INVALID_HANDLE_VALUE {
            let last_error = windows::core::HRESULT::from_win32(0);
            let core_error = Error::from_hresult(last_error);
            let io_error =
                std::io::Error::other(format!("Failed to get {} handle: {}", name, core_error));
            return Err(io_error.into());
        }
        Ok(())
    }

    /// Queries the current console mode for the specified handle.
    ///
    /// Returns the current mode flags or an error if the query fails.
    fn get_console_mode(&self, handle: HANDLE) -> WinResult<CONSOLE_MODE> {
        let mut mode = CONSOLE_MODE::default();
        unsafe { GetConsoleMode(handle, &mut mode)?; }
        Ok(mode)
    }

    /// Sets the console mode for the specified handle.
    ///
    /// Returns an error if the mode setting fails.
    fn set_console_mode(&self, handle: HANDLE, mode: CONSOLE_MODE) -> WinResult<()> {
        unsafe { SetConsoleMode(handle, mode)?; }
        Ok(())
    }

    /// Checks if there is pending input available on stdin without blocking.
    ///
    /// Returns true if input is available, false otherwise.
    fn is_stdin_empty(&self) -> WinResult<bool> {
        // Wait for input with a zero timeout - returns WAIT_OBJECT_0 if input is available
        let wait_result = unsafe { WaitForSingleObject(self.stdin, 0) };
        Ok(wait_result == WAIT_OBJECT_0)
    }

    /// Sets stdin to raw mode, disabling most console processing features.
    ///
    /// Returns an error if the mode setting fails.
    fn set_raw_stdin(&self) -> WinResult<()> {
        let mut mode = self.get_console_mode(self.stdin)?;

        // Disable unwanted features
        mode &= !ENABLE_ECHO_INPUT;
        mode &= !ENABLE_LINE_INPUT;
        mode &= !ENABLE_MOUSE_INPUT;
        mode &= !ENABLE_PROCESSED_INPUT;

        // Enable desired features
        mode |= ENABLE_EXTENDED_FLAGS;
        mode |= ENABLE_INSERT_MODE;
        mode |= ENABLE_QUICK_EDIT_MODE;
        mode |= ENABLE_VIRTUAL_TERMINAL_INPUT;

        self.set_console_mode(self.stdin, mode)
    }
}

fn main() -> WinResult<()> {
    println!("=== Windows Standard Handles Demo ===\n");

    // Create a console session with standard handles
    println!("Retrieving standard handles...");
    let console = ConsoleSession::new()?;
    println!("✓ Successfully retrieved stdin, stdout, and stderr handles\n");

    // Display handle values
    println!("Handle values:");
    println!("  stdin:  0x{:X}", console.stdin.0 as usize);
    println!("  stdout: 0x{:X}", console.stdout.0 as usize);
    println!("  stderr: 0x{:X}\n", console.stderr.0 as usize);

    // Query and display current console modes
    println!("Current console modes:");
    let stdin_mode = console.get_console_mode(console.stdin)?;
    println!("  stdin mode: 0x{:08X}", stdin_mode.0);

    let stdout_mode = console.get_console_mode(console.stdout)?;
    println!("  stdout mode: 0x{:08X}", stdout_mode.0);

    let stderr_mode = console.get_console_mode(console.stderr)?;
    println!("  stderr mode: 0x{:08X}\n", stderr_mode.0);

    // Check for pending input
    println!("Checking for pending input on stdin...");
    match console.is_stdin_empty() {
        Ok(true) => println!("✓ Input is available on stdin\n"),
        Ok(false) => println!("✓ No input available on stdin\n"),
        Err(e) => {
            eprintln!("Error checking stdin: {}\n", e);
        }
    }

    // Demonstrate setting raw mode
    println!("Setting stdin to raw mode...");
    match console.set_raw_stdin() {
        Ok(_) => println!("✓ Successfully set stdin to raw mode\n"),
        Err(e) => {
            eprintln!("Warning: Failed to set raw mode: {}\n", e);
        }
    }

    // Restore original mode
    println!("Restoring original stdin mode...");
    match console.set_console_mode(console.stdin, stdin_mode) {
        Ok(_) => println!("✓ Successfully restored original stdin mode\n"),
        Err(e) => {
            eprintln!("Warning: Failed to restore stdin mode: {}\n", e);
        }
    }

    // Final message
    println!("=== Demo completed successfully ===");

    Ok(())
}
