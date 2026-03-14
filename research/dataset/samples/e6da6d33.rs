use windows::core::{w, Result};
use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

fn main() -> Result<()> {
    let drive_path = w!("C:\\");
    let mut free_bytes_available = 0u64;
    let mut total_bytes = 0u64;
    let mut total_free_bytes = 0u64;

    // SAFETY: `drive_path` is a valid wide string literal and the mutable variables are properly initialized.
    unsafe {
        GetDiskFreeSpaceExW(
            drive_path,
            Some(&mut free_bytes_available),
            Some(&mut total_bytes),
            Some(&mut total_free_bytes),
        )?;
    }

    println!("Disk free space for C:\\:");
    println!("  Free bytes available to caller: {}", free_bytes_available);
    println!("  Total bytes on disk: {}", total_bytes);
    println!("  Total free bytes on disk: {}", total_free_bytes);

    Ok(())
}
