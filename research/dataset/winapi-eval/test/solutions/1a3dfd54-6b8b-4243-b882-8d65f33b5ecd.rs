use windows::core::Result;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Threading::SetEvent;

pub fn signal_event(event: HANDLE) -> Result<()> {
    unsafe { SetEvent(event) }
}
