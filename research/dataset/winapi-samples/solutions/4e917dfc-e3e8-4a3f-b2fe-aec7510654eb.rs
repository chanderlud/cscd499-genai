use windows::core::Result;
use windows::Win32::System::Console;
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;

fn main() -> Result<()> {
    // Allocate a new console for the process
    unsafe {
        Console::AllocConsole()?;
    }

    println!("Console allocated successfully!");

    // Get the current executable's file path
    let mut buffer = [0u16; 260];
    let len = unsafe { GetModuleFileNameW(None, &mut buffer) };

    if len == 0 {
        // If GetModuleFileNameW fails, get the error from GetLastError
        return Err(windows::core::Error::from_thread());
    }

    // Convert the wide string to a Rust String
    let exe_path = String::from_utf16_lossy(&buffer[..len as usize]);
    println!("Current executable path: {}", exe_path);

    Ok(())
}
