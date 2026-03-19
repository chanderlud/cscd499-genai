use windows::core::Result;
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

fn main() -> Result<()> {
    let mut power_status = SYSTEM_POWER_STATUS::default();
    unsafe {
        GetSystemPowerStatus(&mut power_status)?;
    }

    println!("✓ Successfully retrieved system power status\n");

    // AC line status
    let ac_status = if power_status.ACLineStatus == 1 {
        "Online (AC power connected)"
    } else {
        "Offline (battery power)"
    };
    println!("AC Line Status: {}", ac_status);

    // Battery presence
    let battery_present = if power_status.BatteryFlag != 0 {
        "Battery present"
    } else {
        "No battery present"
    };
    println!("Battery Present: {}", battery_present);

    // Battery life percentage
    let battery_percent = match power_status.BatteryLifePercent {
        255 => "Unknown".to_string(),
        p => format!("{}%", p),
    };
    println!("Battery Life: {}", battery_percent);

    // Battery life time in minutes
    let battery_time = if power_status.BatteryLifeTime == 0 {
        "Unknown".to_string()
    } else {
        format!("{} minutes", power_status.BatteryLifeTime)
    };
    println!("Battery Life Time: {}", battery_time);

    // Battery flag details
    let flag_details = match power_status.BatteryFlag {
        1 => "High",
        2 => "Low",
        4 => "Critical",
        8 => "Charging",
        128 => "No system battery",
        _ => "Unknown",
    };
    println!("Battery Flag: {}", flag_details);

    // Summary
    println!("\nPower Status Summary:");
    println!(
        "  - AC Power: {}",
        if power_status.ACLineStatus == 1 {
            "Connected"
        } else {
            "Disconnected"
        }
    );
    println!(
        "  - Battery: {}",
        if power_status.BatteryFlag != 0 {
            "Present"
        } else {
            "Not Present"
        }
    );
    println!("  - Charge Level: {}", battery_percent);
    println!("  - Estimated Runtime: {}", battery_time);

    Ok(())
}
