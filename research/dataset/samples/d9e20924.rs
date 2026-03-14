use windows::core::Result;
use windows::Win32::System::SystemInformation::GetSystemTime;

fn main() -> Result<()> {
    // Get the current system time
    let system_time = unsafe { GetSystemTime() };

    // Print the current system time
    println!("Current system time:");
    println!("  Year: {}", system_time.wYear);
    println!("  Month: {}", system_time.wMonth);
    println!("  Day: {}", system_time.wDay);
    println!("  Day of week: {}", system_time.wDayOfWeek);
    println!("  Hour: {}", system_time.wHour);
    println!("  Minute: {}", system_time.wMinute);
    println!("  Second: {}", system_time.wSecond);
    println!("  Milliseconds: {}", system_time.wMilliseconds);

    Ok(())
}
