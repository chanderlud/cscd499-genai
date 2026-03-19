use windows::core::Result;
use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

fn main() -> Result<()> {
    let mut mem_status = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };

    unsafe { GlobalMemoryStatusEx(&mut mem_status) }?;

    println!("=== System Memory Information ===");
    println!(
        "Total Physical Memory: {} bytes ({:.2} GB)",
        mem_status.ullTotalPhys,
        mem_status.ullTotalPhys as f64 / (1024.0 * 1024.0 * 1024.0)
    );
    println!(
        "Available Physical Memory: {} bytes ({:.2} GB)",
        mem_status.ullAvailPhys,
        mem_status.ullAvailPhys as f64 / (1024.0 * 1024.0 * 1024.0)
    );
    println!(
        "Total Page File: {} bytes ({:.2} GB)",
        mem_status.ullTotalPageFile,
        mem_status.ullTotalPageFile as f64 / (1024.0 * 1024.0 * 1024.0)
    );
    println!(
        "Available Page File: {} bytes ({:.2} GB)",
        mem_status.ullAvailPageFile,
        mem_status.ullAvailPageFile as f64 / (1024.0 * 1024.0 * 1024.0)
    );
    println!(
        "Total Virtual Memory: {} bytes ({:.2} GB)",
        mem_status.ullTotalVirtual,
        mem_status.ullTotalVirtual as f64 / (1024.0 * 1024.0 * 1024.0)
    );
    println!(
        "Available Virtual Memory: {} bytes ({:.2} GB)",
        mem_status.ullAvailVirtual,
        mem_status.ullAvailVirtual as f64 / (1024.0 * 1024.0 * 1024.0)
    );

    let total_phys = mem_status.ullTotalPhys as f64;
    let avail_phys = mem_status.ullAvailPhys as f64;
    let used_phys = total_phys - avail_phys;
    let used_phys_percent = (used_phys / total_phys) * 100.0;

    println!("Memory Usage Summary:");
    println!(
        "  Physical Memory: {:.2}% used ({:.2} GB used / {:.2} GB total)",
        used_phys_percent,
        used_phys / (1024.0 * 1024.0 * 1024.0),
        total_phys / (1024.0 * 1024.0 * 1024.0)
    );

    let total_virtual = mem_status.ullTotalVirtual as f64;
    let avail_virtual = mem_status.ullAvailVirtual as f64;
    let used_virtual = total_virtual - avail_virtual;
    let used_virtual_percent = (used_virtual / total_virtual) * 100.0;

    println!(
        "  Virtual Memory: {:.2}% used ({:.2} GB used / {:.2} GB total)",
        used_virtual_percent,
        used_virtual / (1024.0 * 1024.0 * 1024.0),
        total_virtual / (1024.0 * 1024.0 * 1024.0)
    );

    Ok(())
}
