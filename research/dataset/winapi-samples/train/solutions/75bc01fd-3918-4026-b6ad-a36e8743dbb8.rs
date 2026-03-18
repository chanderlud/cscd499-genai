// Query memory region of the current module

use windows::{
    core::Result,
    Win32::{
        Foundation::HMODULE,
        System::{
            LibraryLoader::GetModuleHandleW,
            Memory::{VirtualQuery, MEMORY_BASIC_INFORMATION},
        },
    },
};

fn query_memory_region(addr: usize) -> Result<MEMORY_BASIC_INFORMATION> {
    let mut mbi: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
    let mbi_size = std::mem::size_of::<MEMORY_BASIC_INFORMATION>();
    // SAFETY: VirtualQuery is called with valid parameters and mbi is properly initialized
    let result = unsafe { VirtualQuery(Some(addr as _), &mut mbi, mbi_size) };
    if result == 0 {
        Err(windows::core::Error::from_thread())
    } else {
        Ok(mbi)
    }
}

fn main() -> Result<()> {
    // Get the base address of the current module
    let module: HMODULE = unsafe { GetModuleHandleW(None) }?;
    let module_addr = module.0 as usize;

    // Query memory information for the module base address
    let memory_info = query_memory_region(module_addr)?;

    println!("Memory region at base address {:#x}:", module_addr);
    println!("  Base address: {:p}", memory_info.BaseAddress);
    println!("  Allocation base: {:p}", memory_info.AllocationBase);
    println!("  Region size: {} bytes", memory_info.RegionSize);
    println!("  State: {:?}", memory_info.State);
    println!("  Protect: {:?}", memory_info.Protect);

    Ok(())
}
