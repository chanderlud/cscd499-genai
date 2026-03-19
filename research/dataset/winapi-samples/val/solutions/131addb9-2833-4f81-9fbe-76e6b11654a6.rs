use windows::core::{Error, Result};
use windows::Win32::Foundation::{LocalFree, E_INVALIDARG, HLOCAL};
use windows::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
};

pub fn dpapi_roundtrip_verify(data: &[u8], entropy: &[u8]) -> Result<()> {
    // Create CRYPT_INTEGER_BLOB structures on stack
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

    // Encrypt the data
    // SAFETY: We're passing valid pointers to data and entropy blobs.
    // The encrypted_blob will be allocated by CryptProtectData.
    unsafe {
        CryptProtectData(
            &data_blob,
            windows::core::w!(""),
            if entropy.is_empty() {
                None
            } else {
                Some(&entropy_blob as *const _)
            },
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut encrypted_blob,
        )?;
    }

    // Ensure we clean up encrypted_blob.pbData even if decryption fails
    let encrypted_data = encrypted_blob;
    let mut decrypted_blob = CRYPT_INTEGER_BLOB::default();

    // Decrypt the data
    // SAFETY: We're passing valid pointers to encrypted_blob and entropy blobs.
    // The decrypted_blob will be allocated by CryptUnprotectData.
    let decrypt_result = unsafe {
        let result = CryptUnprotectData(
            &encrypted_data,
            None,
            if entropy.is_empty() {
                None
            } else {
                Some(&entropy_blob as *const _)
            },
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut decrypted_blob,
        );

        // Free encrypted data regardless of decryption success
        let _ = LocalFree(Some(HLOCAL(encrypted_data.pbData as *mut std::ffi::c_void)));

        result
    };

    // If decryption failed, return error
    decrypt_result?;

    // Ensure we clean up decrypted_blob.pbData
    let decrypted_data = decrypted_blob;

    // Verify the decrypted data matches original
    let result = if decrypted_data.cbData as usize != data.len() {
        Err(Error::from_hresult(E_INVALIDARG))
    } else {
        // SAFETY: We've verified the lengths match, and both pointers are valid
        let matches = unsafe {
            std::slice::from_raw_parts(decrypted_data.pbData, decrypted_data.cbData as usize)
                == data
        };

        if matches {
            Ok(())
        } else {
            Err(Error::from_hresult(E_INVALIDARG))
        }
    };

    // Free decrypted data
    unsafe {
        let _ = LocalFree(Some(HLOCAL(decrypted_data.pbData as *mut std::ffi::c_void)));
    }

    result
}
