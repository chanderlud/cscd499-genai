use windows::core::{Error, Result};
use windows::Win32::Foundation::NTSTATUS;
use windows::Win32::Security::Cryptography::{
    BCryptCreateHash, BCRYPT_ALG_HANDLE, BCRYPT_HASH_HANDLE,
};

fn call_b_crypt_create_hash() -> Result<NTSTATUS> {
    let mut hash_handle = BCRYPT_HASH_HANDLE(std::ptr::null_mut());
    let status = unsafe {
        BCryptCreateHash(
            BCRYPT_ALG_HANDLE(std::ptr::null_mut()),
            &mut hash_handle,
            None,
            None,
            0,
        )
    };
    status.ok()?;
    Ok(status)
}
