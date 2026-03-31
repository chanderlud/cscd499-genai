use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Security::Cryptography::{
    BCryptCreateHash, BCRYPT_ALG_HANDLE, BCRYPT_HASH_HANDLE,
};

fn call_b_crypt_create_hash() -> WIN32_ERROR {
    let mut hash_handle = BCRYPT_HASH_HANDLE::default();
    let ntstatus = unsafe {
        BCryptCreateHash(
            BCRYPT_ALG_HANDLE::default(),
            &mut hash_handle,
            None,
            None,
            0,
        )
    };
    WIN32_ERROR(ntstatus.0 as u32)
}
