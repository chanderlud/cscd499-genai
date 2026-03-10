use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Result, PCWSTR};
use windows::Win32::System::Services::{
    CloseServiceHandle, OpenSCManagerW, OpenServiceW, QueryServiceStatus, SC_MANAGER_CONNECT,
    SERVICE_QUERY_STATUS, SERVICE_RUNNING, SERVICE_STATUS, SERVICE_STOPPED,
};

/// Opens the Service Control Manager for the local computer.
fn open_service_control_manager() -> Result<()> {
    // Open the Service Control Manager with connect access
    let scm_handle = unsafe { OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT)? };

    println!("Service Control Manager opened successfully.");

    // Close the handle when done
    unsafe { CloseServiceHandle(scm_handle)? };

    Ok(())
}

/// Opens a specific service and queries its status.
fn query_service_status(service_name: &str) -> Result<()> {
    // Open the Service Control Manager
    let scm_handle = unsafe { OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT)? };

    // Convert service name to wide string
    let service_name_wide: Vec<u16> = OsStr::new(service_name)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // Open the service with query status access
    let service_handle = unsafe {
        OpenServiceW(
            scm_handle,
            PCWSTR(service_name_wide.as_ptr()),
            SERVICE_QUERY_STATUS,
        )?
    };

    println!("Service '{}' opened successfully.", service_name);

    // Allocate buffer for service status
    let mut status = SERVICE_STATUS::default();

    // Query the service status
    unsafe {
        QueryServiceStatus(service_handle, &mut status)?;
    }

    // Print the current state
    match status.dwCurrentState {
        SERVICE_RUNNING => println!("Service '{}' is RUNNING.", service_name),
        SERVICE_STOPPED => println!("Service '{}' is STOPPED.", service_name),
        other => println!("Service '{}' is in state: {:?}", service_name, other),
    }

    // Close handles safely
    unsafe { CloseServiceHandle(service_handle)? };
    unsafe { CloseServiceHandle(scm_handle)? };

    Ok(())
}

fn main() -> Result<()> {
    // Example 1: Open the Service Control Manager
    println!("=== Example 1: Open Service Control Manager ===");
    open_service_control_manager()?;
    println!();

    // Example 2: Query status of a common service (e.g., "EventLog")
    let service_name = "EventLog"; // Change this to any service name
    query_service_status(service_name)?;
    println!();

    Ok(())
}
