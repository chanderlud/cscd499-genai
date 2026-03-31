use std::ptr;
use windows::core::BOOL;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Security::{AccessCheckAndAuditAlarmA, PSECURITY_DESCRIPTOR};

fn call_access_check_and_audit_alarm_a() -> WIN32_ERROR {
    let mut granted_access: u32 = 0;
    let mut access_status = BOOL(0);
    let mut generate_on_close = BOOL(0);

    // SAFETY: We pass valid mutable references for output parameters and static string literals for inputs.
    // The API will validate the parameters and return an error, which we catch and convert.
    let result = unsafe {
        AccessCheckAndAuditAlarmA(
            windows::core::s!("TestSubsystem"),
            None,
            windows::core::s!("TestObjectType"),
            windows::core::s!("TestObjectName"),
            PSECURITY_DESCRIPTOR(ptr::null_mut()),
            0,
            ptr::null(),
            false,
            &mut granted_access,
            &mut access_status,
            &mut generate_on_close,
        )
    };

    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
