use windows::{
    core::Result,
    Win32::System::{
        LibraryLoader::GetModuleHandleW,
        Memory::{VirtualQuery, MEMORY_BASIC_INFORMATION, MEM_COMMIT, PAGE_EXECUTE_READ},
    },
};

/// Gets some basic info about a memory location
fn query_memory_region(addr: usize) -> Result<MEMORY_BASIC_INFORMATION> {
    let mut mbi: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
    let mbi_size = std::mem::size_of::<MEMORY_BASIC_INFORMATION>();

    // VirtualQuery returns 0 on failure
    let result = unsafe { VirtualQuery(Some(addr as _), &mut mbi, mbi_size) };
    if result == 0 {
        // Capture GetLastError() as a windows::core::Error
        return Err(windows::core::Error::from_thread());
    }

    Ok(mbi)
}

/// Finds the first memory region after the base address that allows execution
fn get_first_executable_memory_region(base_address: usize) -> Option<(usize, usize)> {
    let mut address = base_address;

    while let Ok(mbi) = query_memory_region(address) {
        if mbi.State == MEM_COMMIT && mbi.Protect.contains(PAGE_EXECUTE_READ) {
            return Some((address, mbi.RegionSize));
        }
        address += mbi.RegionSize;
    }

    None
}

fn main() -> Result<()> {
    // Get the current module handle
    let module = unsafe { GetModuleHandleW(None)? };
    let base_address = module.0 as usize;

    println!(
        "Scanning for executable memory regions starting at base address: 0x{:X}",
        base_address
    );

    match get_first_executable_memory_region(base_address) {
        Some((addr, size)) => {
            println!(
                "Found executable region at 0x{:X} with size {} bytes",
                addr, size
            );
        }
        None => {
            println!("No executable memory regions found");
        }
    }

    Ok(())
}
