use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::{NTSTATUS, STATUS_SUCCESS};
use windows::Win32::Security::Cryptography::{
    BCryptDeriveKey, BCryptDestroyKey, BCryptDestroySecret, BCryptExportKey, BCryptFinalizeKeyPair,
    BCryptGenerateKeyPair, BCryptImportKeyPair, BCryptSecretAgreement, BCRYPT_ALG_HANDLE,
    BCRYPT_ECCPUBLIC_BLOB, BCRYPT_KDF_RAW_SECRET, BCRYPT_KEY_HANDLE, BCRYPT_SECRET_HANDLE,
};

pub fn perform_ecdh_key_agreement(
    alg_handle: BCRYPT_ALG_HANDLE,
    peer_public_key_blob: &[u8],
) -> Result<(Vec<u8>, Vec<u8>)> {
    // Helper to check NTSTATUS and convert to Result
    fn check_status(status: NTSTATUS) -> Result<()> {
        if status != STATUS_SUCCESS {
            Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)))
        } else {
            Ok(())
        }
    }

    // 1. Generate ephemeral ECDH key pair
    let mut key_handle = BCRYPT_KEY_HANDLE::default();
    unsafe {
        check_status(BCryptGenerateKeyPair(
            alg_handle,
            &mut key_handle,
            256, // Key length in bits for curve 25519
            0,
        ))?;
        check_status(BCryptFinalizeKeyPair(key_handle, 0))?;
    }

    // 2. Export public key blob
    let mut public_blob_size = 0u32;
    unsafe {
        check_status(BCryptExportKey(
            key_handle,
            None,
            BCRYPT_ECCPUBLIC_BLOB,
            None,
            &mut public_blob_size,
            0,
        ))?;
    }

    let mut public_blob = vec![0u8; public_blob_size as usize];
    unsafe {
        check_status(BCryptExportKey(
            key_handle,
            None,
            BCRYPT_ECCPUBLIC_BLOB,
            Some(public_blob.as_mut_slice()),
            &mut public_blob_size,
            0,
        ))?;
    }

    // 3. Import peer's public key
    let mut peer_key_handle = BCRYPT_KEY_HANDLE::default();
    unsafe {
        check_status(BCryptImportKeyPair(
            alg_handle,
            None,
            BCRYPT_ECCPUBLIC_BLOB,
            &mut peer_key_handle,
            peer_public_key_blob,
            0,
        ))?;
    }

    // 4. Derive shared secret
    let mut secret_handle = BCRYPT_SECRET_HANDLE::default();
    unsafe {
        check_status(BCryptSecretAgreement(
            key_handle,
            peer_key_handle,
            &mut secret_handle,
            0,
        ))?;
    }

    // Get size of derived key
    let mut derived_key_size = 0u32;
    unsafe {
        check_status(BCryptDeriveKey(
            secret_handle,
            BCRYPT_KDF_RAW_SECRET,
            None,
            None,
            &mut derived_key_size,
            0,
        ))?;
    }

    // Derive the raw shared secret
    let mut shared_secret = vec![0u8; derived_key_size as usize];
    unsafe {
        check_status(BCryptDeriveKey(
            secret_handle,
            BCRYPT_KDF_RAW_SECRET,
            None,
            Some(shared_secret.as_mut_slice()),
            &mut derived_key_size,
            0,
        ))?;
    }

    // Clean up BCrypt handles
    unsafe {
        let _ = BCryptDestroyKey(key_handle);
        let _ = BCryptDestroyKey(peer_key_handle);
        let _ = BCryptDestroySecret(secret_handle);
    }

    Ok((public_blob, shared_secret))
}

fn main() -> Result<()> {
    // Example usage - this would need actual values in a real application
    let alg_handle = BCRYPT_ALG_HANDLE::default();
    let peer_public_key = vec![0u8; 32]; // Example placeholder
    let (public_blob, shared_secret) = perform_ecdh_key_agreement(alg_handle, &peer_public_key)?;
    println!("Public blob: {:?}", public_blob);
    println!("Shared secret: {:?}", shared_secret);
    Ok(())
}
