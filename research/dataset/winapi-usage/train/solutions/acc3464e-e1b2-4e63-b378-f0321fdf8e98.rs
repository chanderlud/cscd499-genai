use windows::core::{Error, Result};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::AddLogContainer;

#[allow(dead_code)]
fn call_add_log_container() -> Result<()> {
    unsafe {
        AddLogContainer(
            HANDLE::default(),
            None,
            windows::core::w!("container.log"),
            None,
        )?;
        Ok(())
    }
}
