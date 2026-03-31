use windows::Win32::System::Threading::GetCurrentProcessId;

pub fn current_process_id() -> u32 {
    unsafe { GetCurrentProcessId() }
}
