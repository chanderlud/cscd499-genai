use windows::core::{Error, Owned, Result};
use windows::Win32::Foundation::E_INVALIDARG;
use windows::Win32::Security::Cryptography::{
    BCryptImportKey, BCRYPT_ALG_HANDLE, BCRYPT_KEY_DATA_BLOB, BCRYPT_KEY_DATA_BLOB_HEADER,
    BCRYPT_KEY_DATA_BLOB_MAGIC, BCRYPT_KEY_DATA_BLOB_VERSION1, BCRYPT_KEY_HANDLE,
};

pub fn import_aes_key(
    alg_handle: BCRYPT_ALG_HANDLE,
    key_data: &[u8],
) -> Result<Owned<BCRYPT_KEY_HANDLE>> {
    // Validate key length
    let key_len = key_data.len();
    if key_len != 16 && key_len != 24 && key_len != 32 {
        return Err(Error::from_hresult(E_INVALIDARG));
    }

    // Construct BCRYPT_KEY_DATA_BLOB header
    let header = BCRYPT_KEY_DATA_BLOB_HEADER {
        dwMagic: BCRYPT_KEY_DATA_BLOB_MAGIC,
        dwVersion: BCRYPT_KEY_DATA_BLOB_VERSION1,
        cbKeyData: key_len as u32,
    };

    // Create buffer with header followed by key data
    let header_size = std::mem::size_of::<BCRYPT_KEY_DATA_BLOB_HEADER>();
    let mut blob = vec![0u8; header_size + key_len];

    // SAFETY: We're writing the header struct into the buffer at a properly aligned location
    unsafe {
        std::ptr::write_unaligned(
            blob.as_mut_ptr() as *mut BCRYPT_KEY_DATA_BLOB_HEADER,
            header,
        );
    }
    blob[header_size..].copy_from_slice(key_data);

    // Import the key
    let mut key_handle = BCRYPT_KEY_HANDLE::default();

    // SAFETY: BCryptImportKey is called with valid parameters:
    // - alg_handle is provided by caller and assumed valid
    // - We're importing from BCRYPT_KEY_DATA_BLOB format
    // - blob contains valid header + key data
    // - key_handle is a valid out parameter
    let result = unsafe {
        BCryptImportKey(
            alg_handle,
            None,
            BCRYPT_KEY_DATA_BLOB,
            &mut key_handle,
            None,
            &blob,
            0,
        )
    };

    // Convert NTSTATUS to Result using .ok() which handles the conversion to HRESULT
    result.ok()?;

    // SAFETY: key_handle was successfully created by BCryptImportKey
    // We wrap it in Owned for automatic cleanup
    Ok(unsafe { Owned::new(key_handle) })
}
