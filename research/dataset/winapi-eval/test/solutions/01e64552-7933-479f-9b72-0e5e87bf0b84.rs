use windows::Win32::Foundation::FILETIME;
use windows::Win32::System::SystemInformation::GetSystemTimePreciseAsFileTime;

pub fn current_system_time_100ns() -> u64 {
    let ft: FILETIME = unsafe { GetSystemTimePreciseAsFileTime() };
    ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64)
}
