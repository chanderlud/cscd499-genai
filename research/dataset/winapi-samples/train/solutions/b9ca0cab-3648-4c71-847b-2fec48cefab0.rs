use windows::core::{Error, PCWSTR};
use windows::Win32::Foundation::ERROR_INVALID_DATA;
use windows::Win32::Security::Cryptography::{
    BCryptExportKey, BCryptGetProperty, BCRYPT_ALGORITHM_NAME, BCRYPT_ECCKEY_BLOB,
    BCRYPT_ECCPUBLIC_BLOB, BCRYPT_KEY_HANDLE, BCRYPT_KEY_LENGTH,
};

// Define magic numbers manually since they're not available in the windows crate
pub const BCRYPT_ECDSA_PUBLIC_MAGIC: u32 = 0x31415350;
pub const BCRYPT_ECDH_PUBLIC_MAGIC: u32 = 0x314B4345;

pub fn export_ecc_public_key_to_uncompressed_point(
    key_handle: BCRYPT_KEY_HANDLE,
) -> windows::core::Result<Vec<u8>> {
    // Query algorithm name to determine key type
    let mut alg_name_buffer = [0u16; 256];
    let mut result_len = 0u32;
    unsafe {
        BCryptGetProperty(
            key_handle.into(), // Convert BCRYPT_KEY_HANDLE to BCRYPT_HANDLE
            BCRYPT_ALGORITHM_NAME,
            Some(&mut *(&mut alg_name_buffer as *mut u16 as *mut [u8; 512])),
            &mut result_len,
            0,
        )
        .ok()?;
    }

    let alg_name = unsafe {
        PCWSTR::from_raw(alg_name_buffer.as_ptr())
            .to_string()
            .map_err(|_| {
                Error::from_hresult(windows::core::HRESULT::from_win32(ERROR_INVALID_DATA.0))
            })?
    };

    let (blob_type, expected_magic) = if alg_name.starts_with("ECDSA") {
        (BCRYPT_ECCPUBLIC_BLOB, BCRYPT_ECDSA_PUBLIC_MAGIC)
    } else if alg_name.starts_with("ECDH") {
        (BCRYPT_ECCPUBLIC_BLOB, BCRYPT_ECDH_PUBLIC_MAGIC)
    } else {
        return Err(Error::from_hresult(windows::core::HRESULT::from_win32(
            ERROR_INVALID_DATA.0,
        )));
    };

    // Query key length in bits
    let mut key_length_bits = 0u32;
    let mut result_len = 0u32;
    unsafe {
        BCryptGetProperty(
            key_handle.into(), // Convert BCRYPT_KEY_HANDLE to BCRYPT_HANDLE
            BCRYPT_KEY_LENGTH,
            Some(&mut *(&mut key_length_bits as *mut u32 as *mut [u8; 4])),
            &mut result_len,
            0,
        )
        .ok()?;
    }

    let key_length_bytes = (key_length_bits / 8) as usize;

    // Export key blob - first get required buffer size
    let mut blob_size = 0u32;
    unsafe {
        BCryptExportKey(key_handle, None, blob_type, None, &mut blob_size, 0).ok()?;
    }

    // Allocate buffer and export
    let mut blob = vec![0u8; blob_size as usize];
    unsafe {
        BCryptExportKey(
            key_handle,
            None,
            blob_type,
            Some(&mut blob),
            &mut blob_size,
            0,
        )
        .ok()?;
    }

    // Validate blob size
    let expected_blob_size = std::mem::size_of::<BCRYPT_ECCKEY_BLOB>() + (key_length_bytes * 2);
    if blob_size as usize != expected_blob_size {
        return Err(Error::from_hresult(windows::core::HRESULT::from_win32(
            ERROR_INVALID_DATA.0,
        )));
    }

    // Parse and validate blob structure
    let ecc_blob = unsafe { &*(blob.as_ptr() as *const BCRYPT_ECCKEY_BLOB) };

    if ecc_blob.dwMagic != expected_magic {
        return Err(Error::from_hresult(windows::core::HRESULT::from_win32(
            ERROR_INVALID_DATA.0,
        )));
    }

    if ecc_blob.cbKey != key_length_bytes as u32 {
        return Err(Error::from_hresult(windows::core::HRESULT::from_win32(
            ERROR_INVALID_DATA.0,
        )));
    }

    // Extract coordinates
    let coords_start = std::mem::size_of::<BCRYPT_ECCKEY_BLOB>();
    let x = &blob[coords_start..coords_start + key_length_bytes];
    let y = &blob[coords_start + key_length_bytes..coords_start + key_length_bytes * 2];

    // Build uncompressed point: 0x04 || X || Y
    let mut uncompressed_point = Vec::with_capacity(1 + key_length_bytes * 2);
    uncompressed_point.push(0x04);
    uncompressed_point.extend_from_slice(x);
    uncompressed_point.extend_from_slice(y);

    Ok(uncompressed_point)
}
