use windows::Win32::Foundation::HWND;
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::{AccessibleObjectFromEvent, IAccessible};

fn call_accessible_object_from_event() -> windows::core::HRESULT {
    let hwnd = HWND::default();
    let dwid = 0u32;
    let dwchildid = 0u32;
    let mut ppacc: Option<IAccessible> = None;
    let mut pvarchild: VARIANT = unsafe { std::mem::zeroed() };

    // Call AccessibleObjectFromEvent with concrete parameter values
    // The API returns windows_core::Result<()>, convert to HRESULT
    match unsafe { AccessibleObjectFromEvent(hwnd, dwid, dwchildid, &mut ppacc, &mut pvarchild) } {
        Ok(_) => windows::core::HRESULT::from_win32(0),
        Err(e) => e.code(),
    }
}
