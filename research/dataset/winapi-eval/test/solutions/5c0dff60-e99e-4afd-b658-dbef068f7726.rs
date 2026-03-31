use windows::Win32::System::SystemInformation::GetTickCount64;

pub fn milliseconds_since_boot() -> u64 {
    unsafe { GetTickCount64() }
}
