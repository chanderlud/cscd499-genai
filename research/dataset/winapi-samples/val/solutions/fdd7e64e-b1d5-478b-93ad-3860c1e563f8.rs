use std::ffi::OsString;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::os::windows::ffi::OsStringExt;
use windows::core::{Result, PWSTR};
use windows::Win32::Foundation::{CloseHandle, ERROR_INSUFFICIENT_BUFFER, WIN32_ERROR};
use windows::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, MIB_TCP6ROW_OWNER_PID, MIB_TCP6TABLE_OWNER_PID, TCP_TABLE_OWNER_PID_ALL,
};
use windows::Win32::Networking::WinSock::AF_INET6;
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};

fn get_process_name_from_pid(pid: u32) -> Option<String> {
    unsafe {
        let handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(h) => h,
            Err(_) => return None,
        };

        let mut size: u32 = 260;
        let mut buffer: Vec<u16> = vec![0; size as usize];

        let result = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut size,
        );

        let _ = CloseHandle(handle);

        if result.is_ok() && size > 0 {
            let os_string = OsString::from_wide(&buffer[..size as usize]);
            let path_str = os_string.to_string_lossy().to_string();

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

        let result = GetExtendedTcpTable(
            None,
            &mut size,
            false,
            AF_INET6.0 as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );

        if WIN32_ERROR(result) != ERROR_INSUFFICIENT_BUFFER {
            println!("No IPv6 TCP connections or error: {}", result);
            return Ok(());
        }

        if size == 0 || size > 100_000_000 {
            println!("Invalid size returned: {}", size);
            return Ok(());
        }

        table = vec![0u8; size as usize];
        let result = GetExtendedTcpTable(
            Some(table.as_mut_ptr() as *mut _),
            &mut size,
            false,
            AF_INET6.0 as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );

        if result != 0 {
            println!("Failed to get TCP table: {}", result);
            return Ok(());
        }

        if table.len() < std::mem::size_of::<u32>() {
            println!("TCP table buffer too small for header");
            return Ok(());
        }

        let tcp_table = &*(table.as_ptr() as *const MIB_TCP6TABLE_OWNER_PID);
        let num_entries = tcp_table.dwNumEntries as usize;

        let required_size =
            std::mem::size_of::<u32>() + num_entries * std::mem::size_of::<MIB_TCP6ROW_OWNER_PID>();
        if table.len() < required_size {
            println!(
                "TCP table buffer too small: got {} bytes, need {} for {} entries",
                table.len(),
                required_size,
                num_entries
            );
            return Ok(());
        }

        println!("Found {} IPv6 TCP connections:", num_entries);

        let rows_ptr = &tcp_table.table[0] as *const MIB_TCP6ROW_OWNER_PID;

        for i in 0..num_entries {
            let row = &*rows_ptr.add(i);

            let local_addr = SocketAddr::new(
                IpAddr::V6(Ipv6Addr::from(row.ucLocalAddr)),
                u16::from_be(row.dwLocalPort as u16),
            );

            let remote_addr = SocketAddr::new(
                IpAddr::V6(Ipv6Addr::from(row.ucRemoteAddr)),
                u16::from_be(row.dwRemotePort as u16),
            );

            let process_name =
                get_process_name_from_pid(row.dwOwningPid).unwrap_or_else(|| "Unknown".to_string());

            println!(
                "  {} -> {} (PID: {}, Process: {})",
                local_addr, remote_addr, row.dwOwningPid, process_name
            );
        }
    }

    Ok(())
}
