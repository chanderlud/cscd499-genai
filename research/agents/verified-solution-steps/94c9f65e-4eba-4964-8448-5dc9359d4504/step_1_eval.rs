use windows::core::{Result, Error, HRESULT, PCWSTR};
use windows::Win32::Foundation::{LocalFree, HLOCAL};
use windows::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPT_INTEGER_BLOB,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

fn encrypt_data(data: &[u8], entropy: &[u8]) -> Result<Vec<u8>> {
    let data_blob = CRYPT_INTEGER_BLOB {
        cbData: data.len() as u32,
        pbData: data.as_ptr() as *mut u8,
    };
    
    let entropy_blob = if entropy.is_empty() {
        CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: std::ptr::null_mut(),
        }
    } else {
        CRYPT_INTEGER_BLOB {
            cbData: entropy.len() as u32,
            pbData: entropy.as_ptr() as *mut u8,
        }
    };
    
    let mut encrypted_blob = CRYPT_INTEGER_BLOB::default();
    
    // SAFETY: We're calling the Win32 API with valid pointers and checking the result.
    let result = unsafe {
        CryptProtectData(
            &data_blob,
            PCWSTR::null(),
            if entropy.is_empty() {
                std::ptr::null()
            } else {
                &entropy_blob
            },
            std::ptr::null(),
            std::ptr::null(),
            0,
            &mut encrypted_blob,
        )
    };
    
    if !result.as_bool() {
        return Err(Error::from_thread());
    }
    
    // SAFETY: We're copying data from a valid pointer with a known length.
    let encrypted_data = unsafe {
        std::slice::from_raw_parts(encrypted_blob.pbData, encrypted_blob.cbData as usize)
    }.to_vec();
    
    // SAFETY: We're freeing memory allocated by CryptProtectData.
    unsafe {
        let _ = LocalFree(HLOCAL(encrypted_blob.pbData as *mut std::ffi::c_void));
    }
    
    Ok(encrypted_data)
}

fn decrypt_data(encrypted: &[u8], entropy: &[u8]) -> Result<Vec<u8>> {
    let encrypted_blob = CRYPT_INTEGER_BLOB {
        cbData: encrypted.len() as u32,
        pbData: encrypted.as_ptr() as *mut u8,
    };
    
    let entropy_blob = if entropy.is_empty() {
        CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: std::ptr::null_mut(),
        }
    } else {
        CRYPT_INTEGER_BLOB {
            cbData: entropy.len() as u32,
            pbData: entropy.as_ptr() as *mut u8,
        }
    };
    
    let mut decrypted_blob = CRYPT_INTEGER_BLOB::default();
    
    // SAFETY: We're calling the Win32 API with valid pointers and checking the result.
    let result = unsafe {
        CryptUnprotectData(
            &encrypted_blob,
            std::ptr::null_mut(),
            if entropy.is_empty() {
                std::ptr::null()
            } else {
                &entropy_blob
            },
            std::ptr::null(),
            std::ptr::null(),
            0,
            &mut decrypted_blob,
        )
    };
    
    if !result.as_bool() {
        return Err(Error::from_thread());
    }
    
    // SAFETY: We're copying data from a valid pointer with a known length.
    let decrypted_data = unsafe {
        std::slice::from_raw_parts(decrypted_blob.pbData, decrypted_blob.cbData as usize)
    }.to_vec();
    
    // SAFETY: We're freeing memory allocated by CryptUnprotectData.
    unsafe {
        let _ = LocalFree(HLOCAL(decrypted_blob.pbData as *mut std::ffi::c_void));
    }
    
    Ok(decrypted_data)
}

pub fn dpapi_roundtrip(data: &[u8], entropy: &[u8]) -> Result<Vec<u8>> {
    let encrypted = encrypt_data(data, entropy)?;
    let decrypted = decrypt_data(&encrypted, entropy)?;
    Ok(decrypted)
}