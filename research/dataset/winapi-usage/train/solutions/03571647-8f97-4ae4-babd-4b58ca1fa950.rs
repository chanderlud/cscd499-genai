use windows::core::{Error, Result};
use windows::Win32::Security::{AllocateAndInitializeSid, PSID, SID_IDENTIFIER_AUTHORITY};

fn call_allocate_and_initialize_sid() -> Result<Result<()>> {
    let authority = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 5],
    };
    let mut psid = PSID(std::ptr::null_mut());
    // SAFETY: We pass a valid pointer to the authority struct, a subauthority count of 1,
    // and a mutable pointer to a PSID variable to receive the allocated SID.
    Ok(unsafe { AllocateAndInitializeSid(&authority, 1, 0, 0, 0, 0, 0, 0, 0, 0, &mut psid) })
}
