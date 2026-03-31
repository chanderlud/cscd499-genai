use windows::core::{Error, Result, BOOL};
use windows::Win32::Security::{AccessCheckAndAuditAlarmA, GENERIC_MAPPING, PSECURITY_DESCRIPTOR};

fn call_access_check_and_audit_alarm_a() -> Result<()> {
    let mut granted_access: u32 = 0;
    let mut access_status: BOOL = BOOL(0);
    let mut generate_on_close: BOOL = BOOL(0);

    let generic_mapping = GENERIC_MAPPING {
        GenericRead: 0,
        GenericWrite: 0,
        GenericExecute: 0,
        GenericAll: 0,
    };

    let security_descriptor = PSECURITY_DESCRIPTOR(std::ptr::null_mut());

    // SAFETY: All pointers are valid and initialized. The API handles null/zero defaults
    // and returns a standard Result, which we propagate directly.
    unsafe {
        AccessCheckAndAuditAlarmA(
            windows::core::s!("Subsystem"),
            None,
            windows::core::s!("ObjectType"),
            windows::core::s!("ObjectName"),
            security_descriptor,
            0,
            &generic_mapping,
            false,
            &mut granted_access,
            &mut access_status,
            &mut generate_on_close,
        )
    }
}
