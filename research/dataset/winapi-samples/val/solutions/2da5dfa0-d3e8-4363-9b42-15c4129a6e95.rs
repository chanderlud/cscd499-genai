use std::mem::size_of;
use windows::core::{Error, Result};
use windows::Win32::Security::Cryptography::*;

fn generate_ecdh_p256_keypair() -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    // Open algorithm provider for ECDH P-256
    let mut alg_handle = BCRYPT_ALG_HANDLE::default();
    // SAFETY: Calling Win32 API with valid parameters
    unsafe {
        BCryptOpenAlgorithmProvider(
            &mut alg_handle,
            windows::core::w!("ECDH_P256"),
            None,
            BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(0),
        )
        .ok()?;
    }

    // Ensure we clean up the algorithm provider handle
    struct AlgHandle(BCRYPT_ALG_HANDLE);
    impl Drop for AlgHandle {
        fn drop(&mut self) {
            // SAFETY: Handle was successfully opened
            unsafe {
                let _ = BCryptCloseAlgorithmProvider(self.0, 0);
            }
        }
    }
    let alg_handle = AlgHandle(alg_handle);

    // Generate key pair
    let mut key_handle = BCRYPT_KEY_HANDLE::default();
    // SAFETY: Calling Win32 API with valid parameters
    unsafe {
        BCryptGenerateKeyPair(alg_handle.0, &mut key_handle, 256, 0).ok()?;
    }

    // Ensure we clean up the key handle
    struct KeyHandle(BCRYPT_KEY_HANDLE);
    impl Drop for KeyHandle {
        fn drop(&mut self) {
            // SAFETY: Handle was successfully created
            unsafe {
                let _ = BCryptDestroyKey(self.0);
            }
        }
    }
    let key_handle = KeyHandle(key_handle);

    // Finalize the key pair
    // SAFETY: Calling Win32 API with valid parameters
    unsafe {
        BCryptFinalizeKeyPair(key_handle.0, 0).ok()?;
    }

    // Export public key
    let mut public_blob_size = 0u32;
    // SAFETY: First call to get required size
    unsafe {
        BCryptExportKey(
            key_handle.0,
            None,
            BCRYPT_ECCPUBLIC_BLOB,
            None,
            &mut public_blob_size,
            0,
        )
        .ok()?;
    }

    let mut public_blob = vec![0u8; public_blob_size as usize];
    // SAFETY: Second call to fill the buffer
    unsafe {
        BCryptExportKey(
            key_handle.0,
            None,
            BCRYPT_ECCPUBLIC_BLOB,
            Some(public_blob.as_mut_slice()),
            &mut public_blob_size,
            0,
        )
        .ok()?;
    }

    // Export private key
    let mut private_blob_size = 0u32;
    // SAFETY: First call to get required size
    unsafe {
        BCryptExportKey(
            key_handle.0,
            None,
            BCRYPT_ECCPRIVATE_BLOB,
            None,
            &mut private_blob_size,
            0,
        )
        .ok()?;
    }

    let mut private_blob = vec![0u8; private_blob_size as usize];
    // SAFETY: Second call to fill the buffer
    unsafe {
        BCryptExportKey(
            key_handle.0,
            None,
            BCRYPT_ECCPRIVATE_BLOB,
            Some(private_blob.as_mut_slice()),
            &mut private_blob_size,
            0,
        )
        .ok()?;
    }

    // Parse public key blob
    // BCRYPT_ECCPUBLIC_BLOB format:
    // - BCRYPT_ECCKEY_BLOB header (dwMagic, cbKey)
    // - X coordinate (cbKey bytes)
    // - Y coordinate (cbKey bytes)
    if public_blob.len() < size_of::<BCRYPT_ECCKEY_BLOB>() {
        return Err(Error::from_hresult(
            windows::Win32::Foundation::E_INVALIDARG,
        ));
    }

    // SAFETY: We have enough bytes for the header
    let header =
        unsafe { std::ptr::read_unaligned(public_blob.as_ptr() as *const BCRYPT_ECCKEY_BLOB) };

    if header.dwMagic != BCRYPT_ECDH_PUBLIC_P256_MAGIC {
        return Err(Error::from_hresult(
            windows::Win32::Foundation::E_INVALIDARG,
        ));
    }

    let key_size = header.cbKey as usize;
    let expected_public_size = size_of::<BCRYPT_ECCKEY_BLOB>() + 2 * key_size;
    if public_blob.len() < expected_public_size {
        return Err(Error::from_hresult(
            windows::Win32::Foundation::E_INVALIDARG,
        ));
    }

    let x_start = size_of::<BCRYPT_ECCKEY_BLOB>();
    let y_start = x_start + key_size;
    let x = public_blob[x_start..x_start + key_size].to_vec();
    let y = public_blob[y_start..y_start + key_size].to_vec();

    // Parse private key blob
    // BCRYPT_ECCPRIVATE_BLOB format:
    // - BCRYPT_ECCKEY_BLOB header (dwMagic, cbKey)
    // - X coordinate (cbKey bytes)
    // - Y coordinate (cbKey bytes)
    // - Private scalar (cbKey bytes)
    if private_blob.len() < size_of::<BCRYPT_ECCKEY_BLOB>() {
        return Err(Error::from_hresult(
            windows::Win32::Foundation::E_INVALIDARG,
        ));
    }

    // SAFETY: We have enough bytes for the header
    let private_header =
        unsafe { std::ptr::read_unaligned(private_blob.as_ptr() as *const BCRYPT_ECCKEY_BLOB) };

    if private_header.dwMagic != BCRYPT_ECDH_PRIVATE_P256_MAGIC {
        return Err(Error::from_hresult(
            windows::Win32::Foundation::E_INVALIDARG,
        ));
    }

    let private_key_size = private_header.cbKey as usize;
    let expected_private_size = size_of::<BCRYPT_ECCKEY_BLOB>() + 3 * private_key_size;
    if private_blob.len() < expected_private_size {
        return Err(Error::from_hresult(
            windows::Win32::Foundation::E_INVALIDARG,
        ));
    }

    let scalar_start = size_of::<BCRYPT_ECCKEY_BLOB>() + 2 * private_key_size;
    let private_scalar = private_blob[scalar_start..scalar_start + private_key_size].to_vec();

    Ok((x, y, private_scalar))
}

fn main() -> Result<()> {
    let (x, y, private) = generate_ecdh_p256_keypair()?;
    println!("X: {:x?}", x);
    println!("Y: {:x?}", y);
    println!("Private: {:x?}", private);
    Ok(())
}
