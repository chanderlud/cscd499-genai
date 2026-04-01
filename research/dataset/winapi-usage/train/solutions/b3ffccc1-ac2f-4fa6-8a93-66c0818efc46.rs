use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::RestartManager::RmGetList;

fn call_rm_get_list() -> Result<WIN32_ERROR> {
    let mut proc_info_needed: u32 = 0;
    let mut proc_info: u32 = 0;
    let mut reboot_reasons: u32 = 0;

    let result = unsafe {
        RmGetList(
            0,
            &mut proc_info_needed,
            &mut proc_info,
            None,
            &mut reboot_reasons,
        )
    };

    // Check if the WIN32_ERROR indicates success (0 = ERROR_SUCCESS)
    if result.0 == 0 {
        Ok(result)
    } else {
        // Convert WIN32_ERROR to HRESULT, then to Error
        Err(Error::from_hresult(result.into()))
    }
}
