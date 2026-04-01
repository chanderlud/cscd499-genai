use windows::core::{Error, Result};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Threading::CreateEventA;

fn call_create_event_a() -> Result<HANDLE> {
    // Call CreateEventA with concrete parameter values
    // lpeventattributes: None (no security attributes)
    // bmanualreset: false (auto-reset event)
    // binitialstate: false (initially non-signaled)
    // lpname: None (unnamed event)
    unsafe { CreateEventA(None, false, false, None) }
}
