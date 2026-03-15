// Change memory protection with VirtualProtect and write to memory

use windows::core::Result;
use windows::Win32::System::Memory::{
    VirtualProtect, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS,
};

unsafe fn with_mut_ref<T, F: FnOnce(&mut T)>(address: usize, block: F) -> Result<()> {
    let mut existing_flags: PAGE_PROTECTION_FLAGS = std::mem::zeroed();

    // Change memory protection to allow writing
    VirtualProtect(
        address as *const _,
        std::mem::size_of::<T>(),
        PAGE_EXECUTE_READWRITE,
        &mut existing_flags,
    )?;

    // Execute the closure with mutable reference
    let value = &mut *(address as *mut T);
    block(value);

    // Restore original memory protection
    VirtualProtect(
        address as *const _,
        std::mem::size_of::<T>(),
        existing_flags,
        &mut existing_flags,
    )?;

    Ok(())
}

unsafe fn write<T: Sized>(address: usize, value: T) -> Result<()> {
    with_mut_ref(address, |reference| {
        *reference = value;
    })
}

fn main() -> Result<()> {
    // Example: Write to a static variable by temporarily changing memory protection
    static mut COUNTER: u32 = 42;

    unsafe {
        println!("Original value: {}", COUNTER);

        // Write new value using VirtualProtect to change protection
        write(&mut COUNTER as *mut u32 as usize, 100)?;

        println!("New value: {}", COUNTER);
    }

    Ok(())
}
