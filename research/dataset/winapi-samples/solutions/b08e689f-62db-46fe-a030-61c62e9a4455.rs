// Enumerate TCP Connections and Owning Processes using IP Helper API

use std::ffi::OsString;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::os::windows::ffi::OsStringExt;
use windows::core::{Error, Result, PWSTR};
use windows::Win32::Foundation::{CloseHandle, ERROR_INSUFFICIENT_BUFFER, WIN32_ERROR};
use windows::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, MIB_TCPROW_OWNER_PID, MIB_TCPTABLE_OWNER_PID, TCP_TABLE_OWNER_PID_ALL,
};
use windows::Win32::Networking::WinSock::AF_INET;
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};

fn get_process_name_from_pid(pid: u32) -> Option<String> {
    unsafe {
        // Open process with query information access
        let handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(h) => h,
            Err(_) => return None,
        };

        // Query process image name
        let mut size: u32 = 260; // MAX_PATH
        let mut buffer: Vec<u16> = vec![0; size as usize];

        let result = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut size,
        );

        let _ = CloseHandle(handle);

        if result.is_ok() && size > 0 {
            // Convert to OsString and then to String
            let os_string = OsString::from_wide(&buffer[..size as usize]);
            let path_str = os_string.to_string_lossy().to_string();

            // Extract just the filename
            if let Some(filename) = path_str.split('\\').next_back() {
                return Some(filename.to_string());
            }
        }

        None
    }
}

fn main() -> Result<()> {
    unsafe {
        let mut size: u32 = 0;
        let mut table: Vec<u8>;

        // First call to get buffer size
        let result = GetExtendedTcpTable(
            None,
            &mut size,
            false,
            AF_INET.0 as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );

        if WIN32_ERROR(result) != ERROR_INSUFFICIENT_BUFFER {
            println!("No TCP connections or error occurred");
            return Ok(());
        }

        if size == 0 {
            println!("No TCP connections found");
            return Ok(());
        }

        // Allocate buffer and get actual data
        table = vec![0u8; size as usize];
        let result = GetExtendedTcpTable(
            Some(table.as_mut_ptr() as *mut _),
            &mut size,
            false,
            AF_INET.0 as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );

        if result != 0 {
            return Err(Error::from_hresult(windows::core::HRESULT::from_win32(
                result,
            )));
        }

        // Parse the table
        let tcp_table = &*(table.as_ptr() as *const MIB_TCPTABLE_OWNER_PID);
        let num_entries = tcp_table.dwNumEntries as usize;

        println!("Found {} TCP connections:", num_entries);
        println!(
            "{:<20} {:<20} {:<10} {}",
            "Local Address", "Remote Address", "PID", "Process Name"
        );
        println!("{}", "-".repeat(70));

        // Get pointer to the first entry
        let rows_ptr = &tcp_table.table[0] as *const MIB_TCPROW_OWNER_PID;

        for i in 0..num_entries {
            let row = &*rows_ptr.add(i);

            let local_addr = SocketAddr::new(
                IpAddr::V4(Ipv4Addr::from(row.dwLocalAddr.to_ne_bytes())),
                u16::from_be(row.dwLocalPort as u16),
            );

            let remote_addr = SocketAddr::new(
                IpAddr::V4(Ipv4Addr::from(row.dwRemoteAddr.to_ne_bytes())),
                u16::from_be(row.dwRemotePort as u16),
            );

            let process_name = get_process_name_from_pid(row.dwOwningPid)
                .unwrap_or_else(|| "<unknown>".to_string());

            println!(
                "{:<20} {:<20} {:<10} {}",
                local_addr, remote_addr, row.dwOwningPid, process_name
            );
        }
    }

    Ok(())
}
