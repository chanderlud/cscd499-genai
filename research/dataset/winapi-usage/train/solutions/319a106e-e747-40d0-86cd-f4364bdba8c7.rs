use windows::Win32::Foundation::{GetLastError, ERROR_SUCCESS, WIN32_ERROR};
use windows::Win32::System::Console::AllocConsole;

fn call_alloc_console() -> WIN32_ERROR {
    unsafe {
        match AllocConsole() {
            Ok(()) => ERROR_SUCCESS,
            Err(_) => WIN32_ERROR(GetLastError().0),
        }
    }
}
