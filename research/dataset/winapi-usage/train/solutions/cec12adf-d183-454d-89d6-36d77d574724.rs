use windows::core::{Error, Result};
use windows::Win32::Foundation::{CompareObjectHandles, HANDLE};

fn call_compare_object_handles() -> Result<windows::core::BOOL> {
    let h1 = HANDLE::default();
    let h2 = HANDLE::default();
    let result = unsafe { CompareObjectHandles(h1, h2) };
    result.ok()?;
    Ok(result)
}
