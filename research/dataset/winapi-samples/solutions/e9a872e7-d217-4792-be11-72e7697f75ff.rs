use windows::core::Result;
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::Threading::{
        GetCurrentProcessId, OpenProcess, SetProcessInformation,
        PROCESS_POWER_THROTTLING_CURRENT_VERSION, PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
        PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_INFORMATION,
    },
};

pub struct ProcessHandle(HANDLE);

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

pub fn open_process(pid: u32) -> Result<ProcessHandle> {
    unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SET_INFORMATION,
            false,
            pid,
        )
    }
    .map(ProcessHandle)
}

pub fn disable_ecoqos(handle: &ProcessHandle) -> Result<()> {
    let state = windows::Win32::System::Threading::PROCESS_POWER_THROTTLING_STATE {
        Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
        ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
        StateMask: 0,
    };

    unsafe {
        SetProcessInformation(
            handle.0,
            windows::Win32::System::Threading::ProcessPowerThrottling,
            &state as *const _ as *const std::ffi::c_void,
            std::mem::size_of_val(&state) as u32,
        )?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let pid = unsafe { GetCurrentProcessId() };
    let process_handle = open_process(pid)?;
    disable_ecoqos(&process_handle)?;
    println!("Successfully disabled EcoQoS for current process");
    Ok(())
}
