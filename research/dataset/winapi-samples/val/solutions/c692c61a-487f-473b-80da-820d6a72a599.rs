use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{E_INVALIDARG, STATUS_SUCCESS};
use windows::Win32::Security::Cryptography::*;

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

struct AlgorithmHandle(BCRYPT_ALG_HANDLE);

impl Drop for AlgorithmHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = BCryptCloseAlgorithmProvider(self.0, 0);
        }
    }
}

struct KeyHandle(BCRYPT_KEY_HANDLE);

impl Drop for KeyHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = BCryptDestroyKey(self.0);
        }
    }
}

fn pbkdf2_derive(
    password: &[u8],
    salt: Option<&[u8]>,
    iterations: u32,
    hash_algorithm_id: &[u8],
    key_length: usize,
) -> Result<Vec<u8>> {
    if iterations < 1000 {
        return Err(Error::from_hresult(E_INVALIDARG));
    }

    // Convert algorithm ID to wide string
    let alg_id_str =
        std::str::from_utf8(hash_algorithm_id).map_err(|_| Error::from_hresult(E_INVALIDARG))?;
    let alg_id_wide = wide_null(OsStr::new(alg_id_str));

    // Open algorithm provider
    let mut alg_handle = BCRYPT_ALG_HANDLE::default();
    let status = unsafe {
        BCryptOpenAlgorithmProvider(
            &mut alg_handle,
            PCWSTR(alg_id_wide.as_ptr()),
            None,
            BCRYPT_ALG_HANDLE_HMAC_FLAG,
        )
    };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(status.to_hresult()));
    }
    let alg_handle = AlgorithmHandle(alg_handle);

    // Set iteration count
    let iterations_bytes = iterations.to_le_bytes();
    let status = unsafe {
        BCryptSetProperty(
            alg_handle.0.into(),
            PCWSTR(wide_null(OsStr::new("PBKDF2IterationCount")).as_ptr()),
            &iterations_bytes,
            0,
        )
    };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(status.to_hresult()));
    }

    // Set salt if provided
    if let Some(salt_bytes) = salt {
        let status = unsafe {
            BCryptSetProperty(
                alg_handle.0.into(),
                PCWSTR(wide_null(OsStr::new("PBKDF2Salt")).as_ptr()),
                salt_bytes,
                0,
            )
        };
        if status != STATUS_SUCCESS {
            return Err(Error::from_hresult(status.to_hresult()));
        }
    }

    // Create key from password
    let mut key_handle = BCRYPT_KEY_HANDLE::default();
    let status =
        unsafe { BCryptGenerateSymmetricKey(alg_handle.0, &mut key_handle, None, password, 0) };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(status.to_hresult()));
    }
    let key_handle = KeyHandle(key_handle);

    // Derive key using PBKDF2
    let mut derived_key = vec![0u8; key_length];
    let mut result_length = 0u32;
    let status =
        unsafe { BCryptKeyDerivation(key_handle.0, None, &mut derived_key, &mut result_length, 0) };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(status.to_hresult()));
    }

    // Resize to actual derived length
    derived_key.truncate(result_length as usize);
    Ok(derived_key)
}

fn main() {
    let password = b"password";
    let salt = Some(b"salt".as_ref());
    let iterations = 1000;
    let hash_algorithm_id = b"SHA256";
    let key_length = 32;

    match pbkdf2_derive(password, salt, iterations, hash_algorithm_id, key_length) {
        Ok(derived_key) => println!("Derived key: {:?}", derived_key),
        Err(e) => println!("Error: {:?}", e),
    }
}
