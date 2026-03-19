use windows::core::{Result, PCWSTR};
use windows::Win32::Security::Cryptography::{
    BCryptCloseAlgorithmProvider, BCryptDestroyKey, BCryptExportKey, BCryptFinalizeKeyPair,
    BCryptGenerateKeyPair, BCryptOpenAlgorithmProvider, BCRYPT_ALG_HANDLE, BCRYPT_KEY_HANDLE,
    BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS, BCRYPT_RSAFULLPRIVATE_BLOB, BCRYPT_RSAPUBLIC_BLOB,
    BCRYPT_RSA_ALGORITHM,
};

pub fn generate_rsa_keypair(bit_length: u32) -> Result<(Vec<u8>, Vec<u8>)> {
    // Helper to export a key blob
    fn export_key_blob(key_handle: BCRYPT_KEY_HANDLE, blob_type: PCWSTR) -> Result<Vec<u8>> {
        let mut size = 0u32;

        // First call to get required buffer size
        unsafe {
            BCryptExportKey(key_handle, None, blob_type, None, &mut size, 0).ok()?;
        }

        let mut buffer = vec![0u8; size as usize];

        // Second call to actually export the key
        unsafe {
            BCryptExportKey(
                key_handle,
                None,
                blob_type,
                Some(buffer.as_mut_slice()),
                &mut size,
                0,
            )
            .ok()?;
        }

        buffer.truncate(size as usize);
        Ok(buffer)
    }

    // Open algorithm provider
    let mut alg_handle = BCRYPT_ALG_HANDLE::default();
    unsafe {
        BCryptOpenAlgorithmProvider(
            &mut alg_handle,
            BCRYPT_RSA_ALGORITHM,
            PCWSTR::null(),
            BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS::default(),
        )
        .ok()?;
    }

    // Generate key pair
    let mut key_handle = BCRYPT_KEY_HANDLE::default();
    unsafe {
        BCryptGenerateKeyPair(alg_handle, &mut key_handle, bit_length, 0).ok()?;
    }

    // Finalize key pair
    unsafe {
        BCryptFinalizeKeyPair(key_handle, 0).ok()?;
    }

    // Export both key blobs
    let public_blob = export_key_blob(key_handle, BCRYPT_RSAPUBLIC_BLOB)?;
    let private_blob = export_key_blob(key_handle, BCRYPT_RSAFULLPRIVATE_BLOB)?;

    // Clean up handles
    unsafe {
        BCryptDestroyKey(key_handle).ok()?;
        BCryptCloseAlgorithmProvider(alg_handle, 0).ok()?;
    }

    Ok((public_blob, private_blob))
}
