use windows::core::{Error, Result, HRESULT};
use windows::Win32::Security::Cryptography::{
    BCryptCreateHash, BCRYPT_ALG_HANDLE, BCRYPT_HASH_HANDLE,
};

fn call_b_crypt_create_hash() -> HRESULT {
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
    HRESULT(status.0)
}
