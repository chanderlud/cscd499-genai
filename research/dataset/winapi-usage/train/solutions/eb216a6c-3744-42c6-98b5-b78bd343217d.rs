use windows::core::HRESULT;
use windows::core::{Error, Result};
use windows::Win32::System::RestartManager::{RmGetList, RM_PROCESS_INFO};

fn call_rm_get_list() -> windows::core::HRESULT {
    let session_handle: u32 = 0;
    let mut proc_info_needed: u32 = 0;
    let mut proc_info: u32 = 0;
    let affected_apps: Option<*mut RM_PROCESS_INFO> = None;
    let mut reboot_reasons: u32 = 0;

    let result = unsafe {
        RmGetList(
            session_handle,
            &mut proc_info_needed,
            &mut proc_info,
            affected_apps,
            &mut reboot_reasons,
        )
    };

    if result.0 != 0 {
        HRESULT::from_win32(result.0)
    } else {
        HRESULT::from_win32(0)
    }
}
