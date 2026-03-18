use std::mem::size_of;
use windows::core::{Error, Result};
use windows::Win32::Foundation::E_INVALIDARG;
use windows::Win32::Security::Cryptography::{
    BCryptDestroyKey, BCryptImportKeyPair, BCRYPT_ALG_HANDLE, BCRYPT_DSA_KEY_BLOB_V2,
    BCRYPT_DSA_PRIVATE_MAGIC, BCRYPT_KEY_HANDLE, LEGACY_DSA_V2_PRIVATE_BLOB,
};

/// Wrapper that destroys a BCRYPT_KEY_HANDLE on drop.
pub struct Owned(BCRYPT_KEY_HANDLE);

impl Drop for Owned {
    fn drop(&mut self) {
        // SAFETY: The handle must be valid and we have ownership.
        unsafe {
            let _ = BCryptDestroyKey(self.0);
        }
    }
}

impl std::ops::Deref for Owned {
    type Target = BCRYPT_KEY_HANDLE;
    fn deref(&self) -> &BCRYPT_KEY_HANDLE {
        &self.0
    }
}

fn import_dsa_private_key(
    alg_handle: BCRYPT_ALG_HANDLE,
    prime_modulus: &[u8],
    prime_divisor: &[u8],
    generator: &[u8],
    private_key: &[u8],
) -> Result<Owned> {
    // Validate input constraints
    if prime_divisor.len() != 20 {
        return Err(Error::from_hresult(E_INVALIDARG));
    }
    if private_key.len() != 20 {
        return Err(Error::from_hresult(E_INVALIDARG));
    }
    if generator.len() != prime_modulus.len() {
        return Err(Error::from_hresult(E_INVALIDARG));
    }

    // Calculate total blob size
    let header_size = size_of::<BCRYPT_DSA_KEY_BLOB_V2>();
    let total_size = header_size + (prime_modulus.len() * 2) + 40; // prime + generator + divisor + private key

    // Allocate and initialize the blob
    let mut blob = vec![0u8; total_size];

    // SAFETY: We have enough space and the alignment is correct for the header
    unsafe {
        let header = blob.as_mut_ptr() as *mut BCRYPT_DSA_KEY_BLOB_V2;
        (*header).dwMagic = BCRYPT_DSA_PRIVATE_MAGIC;
        (*header).cbKey = prime_modulus.len() as u32;
        (*header).cbGroupSize = 20;
        (*header).Count = [0; 4];
        // Remove the Seed field - it doesn't exist in BCRYPT_DSA_KEY_BLOB_V2
        // Set cbSeedLength to 0 since we're not providing seed data
        (*header).cbSeedLength = 0;
    }

    // Copy the key material after the header
    let mut offset = header_size;
    blob[offset..offset + prime_modulus.len()].copy_from_slice(prime_modulus);
    offset += prime_modulus.len();
    blob[offset..offset + prime_modulus.len()].copy_from_slice(generator);
    offset += prime_modulus.len();
    blob[offset..offset + 20].copy_from_slice(prime_divisor);
    offset += 20;
    blob[offset..offset + 20].copy_from_slice(private_key);

    // Import the key
    let mut key_handle = BCRYPT_KEY_HANDLE::default();

    // SAFETY: All parameters are valid and we've properly constructed the blob
    unsafe {
        BCryptImportKeyPair(
            alg_handle,
            None,
            LEGACY_DSA_V2_PRIVATE_BLOB,
            &mut key_handle,
            &blob,
            0,
        )
        .ok()?;
    }

    Ok(Owned(key_handle))
}
