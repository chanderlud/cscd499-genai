use std::mem;
use windows::core::{Error, Result, HRESULT};
use windows::Win32::Security::Cryptography::{
    BCryptExportKey, BCRYPT_ECCKEY_BLOB, BCRYPT_ECCPRIVATE_BLOB, BCRYPT_ECDH_PRIVATE_P256_MAGIC,
    BCRYPT_ECDH_PRIVATE_P384_MAGIC, BCRYPT_ECDH_PRIVATE_P521_MAGIC,
    BCRYPT_ECDSA_PRIVATE_P256_MAGIC, BCRYPT_ECDSA_PRIVATE_P384_MAGIC,
    BCRYPT_ECDSA_PRIVATE_P521_MAGIC, BCRYPT_KEY_HANDLE,
};

fn export_ecc_private_key(key_handle: BCRYPT_KEY_HANDLE) -> Result<Vec<u8>> {
    // First call to get required buffer size
    let mut blob_size = 0u32;
    unsafe {
        BCryptExportKey(
            key_handle,
            None,
            BCRYPT_ECCPRIVATE_BLOB,
            None,
            &mut blob_size,
            0, // dwFlags
        )
        .ok()?;
    }

    // Allocate buffer for the key blob
    let mut blob_buffer = vec![0u8; blob_size as usize];

    // Second call to actually export the key
    unsafe {
        BCryptExportKey(
            key_handle,
            None,
            BCRYPT_ECCPRIVATE_BLOB,
            Some(&mut blob_buffer),
            &mut blob_size,
            0, // dwFlags
        )
        .ok()?;
    }

    // Parse the BCRYPT_ECCKEY_BLOB header
    if blob_buffer.len() < mem::size_of::<BCRYPT_ECCKEY_BLOB>() {
        return Err(Error::from_hresult(E_INVALIDARG));
    }

    let header = unsafe { &*(blob_buffer.as_ptr() as *const BCRYPT_ECCKEY_BLOB) };

    // Verify this is an ECC private key by checking against known private key magic values
    let is_private_key = matches!(
        header.dwMagic,
        BCRYPT_ECDSA_PRIVATE_P256_MAGIC
            | BCRYPT_ECDSA_PRIVATE_P384_MAGIC
            | BCRYPT_ECDSA_PRIVATE_P521_MAGIC
            | BCRYPT_ECDH_PRIVATE_P256_MAGIC
            | BCRYPT_ECDH_PRIVATE_P384_MAGIC
            | BCRYPT_ECDH_PRIVATE_P521_MAGIC
    );

    if !is_private_key {
        return Err(Error::from_hresult(E_INVALIDARG));
    }

    let key_length = header.cbKey as usize;
    let expected_total = mem::size_of::<BCRYPT_ECCKEY_BLOB>() + 3 * key_length;

    if blob_buffer.len() < expected_total {
        return Err(Error::from_hresult(E_INVALIDARG));
    }

    // Extract the three components: X, Y, and private scalar (d)
    let offset = mem::size_of::<BCRYPT_ECCKEY_BLOB>();
    let x = &blob_buffer[offset..offset + key_length];
    let y = &blob_buffer[offset + key_length..offset + 2 * key_length];
    let d = &blob_buffer[offset + 2 * key_length..offset + 3 * key_length];

    // Combine into uncompressed format: X, Y, private scalar
    let mut result = Vec::with_capacity(3 * key_length);
    result.extend_from_slice(x);
    result.extend_from_slice(y);
    result.extend_from_slice(d);

    // No need to call BCryptFreeBuffer - Vec manages its own memory
    Ok(result)
}

// Import E_INVALIDARG constant
use windows::Win32::Foundation::E_INVALIDARG;
