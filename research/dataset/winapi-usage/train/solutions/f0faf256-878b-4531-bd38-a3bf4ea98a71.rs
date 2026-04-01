use windows::Win32::Foundation::HWND;
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::{AccessibleObjectFromEvent, IAccessible};

fn call_accessible_object_from_event() -> windows::core::Result<windows::core::Result<()>> {
    // Create concrete parameter values
    let hwnd = HWND::default();
    let dwid = 0;
    let dwchildid = 0;

    // Create mutable pointers for output parameters
    let mut ppacc: Option<IAccessible> = None;
    let mut pvarchild = VARIANT::default();

    // Call the API in an unsafe block (required by the API signature)
    let result = unsafe {
        AccessibleObjectFromEvent(
            hwnd,
            dwid,
            dwchildid,
            &mut ppacc as *mut Option<IAccessible>,
            &mut pvarchild as *mut VARIANT,
        )
    };

    Ok(result)
}
