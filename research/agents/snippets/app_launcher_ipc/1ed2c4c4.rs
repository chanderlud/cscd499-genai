//! System Uptime Query using GetTickCount64
//! Demonstrates retrieving the system uptime in milliseconds

use std::time::Duration;
use windows::core::Result;
use windows::Win32::System::SystemInformation::GetTickCount64;

fn main() -> Result<()> {
    // SAFETY: GetTickCount64 is a safe Windows API that returns a u64.
    let uptime_ms = unsafe { GetTickCount64() };

    println!("System Uptime: {} milliseconds", uptime_ms);

    // Convert to human-readable format
    let uptime = Duration::from_millis(uptime_ms);

    let hours = uptime.as_secs() / 3600;
    let minutes = (uptime.as_secs() % 3600) / 60;
    let seconds = uptime.as_secs() % 60;

    println!("Formatted: {:02}:{:02}:{:02}", hours, minutes, seconds);

    Ok(())
}
