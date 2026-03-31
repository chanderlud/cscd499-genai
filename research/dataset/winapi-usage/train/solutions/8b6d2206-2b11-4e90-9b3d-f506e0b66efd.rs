use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::Security::PSID;
use windows::Win32::System::Threading::AddSIDToBoundaryDescriptor;

fn call_add_sid_to_boundary_descriptor() -> WIN32_ERROR {
    let mut boundary_descriptor = HANDLE(std::ptr::null_mut());
    let required_sid = PSID(std::ptr::null_mut());

    // SAFETY: Calling AddSIDToBoundaryDescriptor with null pointers as concrete values for this exercise.
    // The result is handled and converted to WIN32_ERROR appropriately.
    match unsafe { AddSIDToBoundaryDescriptor(&mut boundary_descriptor, required_sid) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or_default(),
    }
}
