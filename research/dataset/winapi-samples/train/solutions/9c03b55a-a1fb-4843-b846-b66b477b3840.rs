use std::mem::MaybeUninit;
use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::STATUS_SUCCESS;
use windows::Win32::Security::Cryptography::{
    BCryptCloseAlgorithmProvider, BCryptCreateHash, BCryptDestroyHash, BCryptDestroyKey,
    BCryptFinalizeKeyPair, BCryptFinishHash, BCryptGenerateKeyPair, BCryptGetProperty,
    BCryptHashData, BCryptOpenAlgorithmProvider, BCryptSignHash, BCryptVerifySignature,
    BCRYPT_ALG_HANDLE, BCRYPT_HASH_HANDLE, BCRYPT_HASH_LENGTH, BCRYPT_HASH_REUSABLE_FLAG,
    BCRYPT_KEY_HANDLE, BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS, BCRYPT_PAD_PKCS1,
    BCRYPT_PKCS1_PADDING_INFO, BCRYPT_RSA_ALGORITHM, BCRYPT_SHA256_ALGORITHM,
};

// RAII wrapper for BCrypt algorithm handle
struct AlgorithmHandle(BCRYPT_ALG_HANDLE);

impl AlgorithmHandle {
    fn new() -> Result<Self> {
        let mut handle = BCRYPT_ALG_HANDLE::default();
        let status = unsafe {
            BCryptOpenAlgorithmProvider(
                &mut handle,
                BCRYPT_RSA_ALGORITHM,
                PCWSTR::null(),
                BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(0),
            )
        };
        if status != STATUS_SUCCESS {
            return Err(Error::from_hresult(HRESULT::from_nt(status.0)));
        }
        Ok(Self(handle))
    }

    fn as_raw(&self) -> BCRYPT_ALG_HANDLE {
        self.0
    }
}

impl Drop for AlgorithmHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = BCryptCloseAlgorithmProvider(self.0, 0);
        }
    }
}

// RAII wrapper for BCrypt key handle
struct KeyHandle(BCRYPT_KEY_HANDLE);

impl KeyHandle {
    fn as_raw(&self) -> BCRYPT_KEY_HANDLE {
        self.0
    }
}

impl Drop for KeyHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = BCryptDestroyKey(self.0);
        }
    }
}

// RAII wrapper for BCrypt hash handle
struct HashHandle(BCRYPT_HASH_HANDLE);

impl HashHandle {
    fn as_raw(&self) -> BCRYPT_HASH_HANDLE {
        self.0
    }
}

impl Drop for HashHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = BCryptDestroyHash(self.0);
        }
    }
}

fn rsa_sign_and_verify(message: &[u8], key_size: u32) -> Result<bool> {
    // Validate key size
    match key_size {
        1024 | 2048 | 4096 => {}
        _ => return Err(Error::from_hresult(HRESULT::from_win32(87))), // ERROR_INVALID_PARAMETER
    }

    // Open RSA algorithm provider
    let alg_handle = AlgorithmHandle::new()?;

    // Generate RSA key pair
    let mut key_handle = MaybeUninit::<BCRYPT_KEY_HANDLE>::uninit();
    let status =
        unsafe { BCryptGenerateKeyPair(alg_handle.as_raw(), key_handle.as_mut_ptr(), key_size, 0) };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_nt(status.0)));
    }
    let key_handle = KeyHandle(unsafe { key_handle.assume_init() });

    // Finalize the key pair
    let status = unsafe { BCryptFinalizeKeyPair(key_handle.as_raw(), 0) };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_nt(status.0)));
    }

    // Open SHA-256 algorithm provider
    let mut hash_alg_handle = BCRYPT_ALG_HANDLE::default();
    let status = unsafe {
        BCryptOpenAlgorithmProvider(
            &mut hash_alg_handle,
            BCRYPT_SHA256_ALGORITHM,
            PCWSTR::null(),
            BCRYPT_HASH_REUSABLE_FLAG,
        )
    };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_nt(status.0)));
    }
    let hash_alg_handle = AlgorithmHandle(hash_alg_handle);

    // Get hash length
    let mut hash_length = 0u32;
    let mut result_length = 0u32;
    let status = unsafe {
        BCryptGetProperty(
            hash_alg_handle.as_raw().into(),
            BCRYPT_HASH_LENGTH,
            Some(std::slice::from_raw_parts_mut(
                &mut hash_length as *mut u32 as *mut u8,
                std::mem::size_of::<u32>(),
            )),
            &mut result_length,
            0,
        )
    };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_nt(status.0)));
    }

    // Create hash
    let mut hash_handle = MaybeUninit::<BCRYPT_HASH_HANDLE>::uninit();
    let status = unsafe {
        BCryptCreateHash(
            hash_alg_handle.as_raw(),
            hash_handle.as_mut_ptr(),
            None,
            None,
            BCRYPT_HASH_REUSABLE_FLAG.0,
        )
    };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_nt(status.0)));
    }
    let hash_handle = HashHandle(unsafe { hash_handle.assume_init() });

    // Hash the message
    let status = unsafe { BCryptHashData(hash_handle.as_raw(), message, 0) };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_nt(status.0)));
    }

    // Finish hash and get hash value
    let mut hash_value = vec![0u8; hash_length as usize];
    let status = unsafe { BCryptFinishHash(hash_handle.as_raw(), &mut hash_value, 0) };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_nt(status.0)));
    }

    // Create padding info for RSA PKCS#1 v1.5 with SHA-256
    let padding_info = BCRYPT_PKCS1_PADDING_INFO {
        pszAlgId: BCRYPT_SHA256_ALGORITHM,
    };

    // Get signature length
    let mut signature_length = 0u32;
    let status = unsafe {
        BCryptSignHash(
            key_handle.as_raw(),
            Some(&padding_info as *const BCRYPT_PKCS1_PADDING_INFO as *const std::ffi::c_void),
            &hash_value,
            None,
            &mut signature_length,
            BCRYPT_PAD_PKCS1,
        )
    };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_nt(status.0)));
    }

    // Sign the hash
    let mut signature = vec![0u8; signature_length as usize];
    let status = unsafe {
        BCryptSignHash(
            key_handle.as_raw(),
            Some(&padding_info as *const BCRYPT_PKCS1_PADDING_INFO as *const std::ffi::c_void),
            &hash_value,
            Some(&mut signature),
            &mut signature_length,
            BCRYPT_PAD_PKCS1,
        )
    };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_nt(status.0)));
    }

    // Verify the signature
    let status = unsafe {
        BCryptVerifySignature(
            key_handle.as_raw(),
            Some(&padding_info as *const BCRYPT_PKCS1_PADDING_INFO as *const std::ffi::c_void),
            &hash_value,
            &signature,
            BCRYPT_PAD_PKCS1,
        )
    };

    Ok(status == STATUS_SUCCESS)
}

fn main() -> Result<()> {
    let message = b"test message";
    let key_size = 2048;
    let result = rsa_sign_and_verify(message, key_size)?;
    println!("Verification result: {}", result);
    Ok(())
}
