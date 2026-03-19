use windows::core::Result;
use windows::Win32::System::Memory::{
    VirtualProtect, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS,
};

unsafe fn with_mut_ref<T, F: FnOnce(&mut T)>(address: usize, block: F) -> Result<()> {
    let mut existing_flags: PAGE_PROTECTION_FLAGS = std::mem::zeroed();
    VirtualProtect(
        address as *const _,
        std::mem::size_of::<T>(),
        PAGE_EXECUTE_READWRITE,
        &mut existing_flags,
    )?;
    let value = &mut *(address as *mut T);
    block(value);
    VirtualProtect(
        address as *const _,
        std::mem::size_of::<T>(),
        existing_flags,
        &mut existing_flags,
    )?;
    Ok(())
}

unsafe fn write<T: Sized>(address: usize, value: T) -> Result<()> {
    with_mut_ref(address, |reference| *reference = value)
}

fn main() -> Result<()> {
    static mut COUNTER: u32 = 42;

    unsafe {
        let original = core::ptr::read(&raw const COUNTER);
        println!("Original value: {}", original);

        write(&raw mut COUNTER as usize, 100)?;

        let new_val = core::ptr::read(&raw const COUNTER);
        println!("New value: {}", new_val);
    }

    Ok(())
}
