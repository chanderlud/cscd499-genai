// TITLE: Get Network Interface Statistics with IP Helper API

use windows::core::{Error, Result, HRESULT};
use windows::Win32::NetworkManagement::IpHelper::{FreeMibTable, GetIfTable2, MIB_IF_TABLE2};
use windows::Win32::NetworkManagement::Ndis::IfOperStatusUp;

fn get_interface_stats() -> Result<Vec<String>> {
    unsafe {
        let mut table: *mut MIB_IF_TABLE2 = std::ptr::null_mut();

        // Get the interface table - check return value manually
        let result = GetIfTable2(&mut table);
        if result.0 != 0 {
            return Err(Error::from_hresult(result.to_hresult()));
        }

        // Check if table is null and convert to proper error
        let table_ref = table.as_ref().ok_or_else(|| {
            Error::from_hresult(HRESULT::from_win32(0x80004003)) // E_POINTER
        })?;

        let num_entries = table_ref.NumEntries as usize;
        let mut interfaces = Vec::new();

        for i in 0..num_entries {
            let row = &*table_ref.Table.as_ptr().add(i);

            // Convert interface alias (UTF-16) to String
            let name = String::from_utf16_lossy(&row.Alias)
                .trim_end_matches('\0')
                .to_string();

            if !name.is_empty() && row.OperStatus == IfOperStatusUp {
                interfaces.push(format!(
                    "{}: RX={} bytes, TX={} bytes",
                    name, row.InOctets, row.OutOctets
                ));
            }
        }

        // Free the allocated table
        FreeMibTable(table.cast());

        Ok(interfaces)
    }
}

fn main() -> Result<()> {
    let interfaces = get_interface_stats()?;

    if interfaces.is_empty() {
        println!("No active network interfaces found");
    } else {
        println!("Active network interfaces:");
        for interface in interfaces {
            println!("  {}", interface);
        }
    }

    Ok(())
}
