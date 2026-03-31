#![allow(dead_code, unused_imports)]

use windows::core::{Error, Result, BOOL, HRESULT, PCSTR};
use windows::Win32::Security::{AccessCheckAndAuditAlarmA, PSECURITY_DESCRIPTOR};

fn call_access_check_and_audit_alarm_a() -> HRESULT {
    let mut granted_access: u32 = 0;
    let mut access_status: BOOL = BOOL(0);
    let mut pf_generate_on_close: BOOL = BOOL(0);

    let result: Result<()> = unsafe {
        AccessCheckAndAuditAlarmA(
            PCSTR::null(),
            None,
            PCSTR::null(),
            PCSTR::null(),
            PSECURITY_DESCRIPTOR(std::ptr::null_mut()),
            0,
            std::ptr::null(),
            false,
            &mut granted_access,
            &mut access_status,
            &mut pf_generate_on_close,
        )
    };

    match result {
        Ok(()) => HRESULT(0),
        Err(e) => e.code(),
    }
}
