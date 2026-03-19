use windows::core::{Error, Owned, Result, HRESULT};
use windows::Win32::Security::Cryptography::{
    BCryptImportKeyPair, BCRYPT_ALG_HANDLE, BCRYPT_DH_KEY_BLOB, BCRYPT_DH_PRIVATE_BLOB,
    BCRYPT_DH_PRIVATE_MAGIC, BCRYPT_KEY_HANDLE,
};

pub fn import_dh_private_key(
    alg_handle: BCRYPT_ALG_HANDLE,
    p: &[u8],
    g: &[u8],
    y: &[u8],
    x: &[u8],
) -> Result<Owned<BCRYPT_KEY_HANDLE>> {
    // Validate all input slices have the same length
    let key_size = p.len();
    if g.len() != key_size || y.len() != key_size || x.len() != key_size {
        return Err(Error::from_hresult(HRESULT::from_win32(0x80070057))); // E_INVALIDARG
    }

    // Calculate total blob size: header + 4 key components
    let blob_size = std::mem::size_of::<BCRYPT_DH_KEY_BLOB>() + (4 * key_size);
    let mut blob = vec![0u8; blob_size];

    // Construct the BCRYPT_DH_KEY_BLOB header
    let header = BCRYPT_DH_KEY_BLOB {
        dwMagic: BCRYPT_DH_PRIVATE_MAGIC,
        cbKey: key_size as u32,
    };

    // Copy header to blob
    unsafe {
        let header_ptr = blob.as_mut_ptr() as *mut BCRYPT_DH_KEY_BLOB;
        std::ptr::write(header_ptr, header);
    }

    // Copy key components after header in order: p, g, y, x
    let offset = std::mem::size_of::<BCRYPT_DH_KEY_BLOB>();
    blob[offset..offset + key_size].copy_from_slice(p);
    blob[offset + key_size..offset + 2 * key_size].copy_from_slice(g);
    blob[offset + 2 * key_size..offset + 3 * key_size].copy_from_slice(y);
    blob[offset + 3 * key_size..offset + 4 * key_size].copy_from_slice(x);

    // Import the key
    let mut key_handle = BCRYPT_KEY_HANDLE::default();
    let status = unsafe {
        BCryptImportKeyPair(
            alg_handle,
            None,
            BCRYPT_DH_PRIVATE_BLOB,
            &mut key_handle,
            &blob,
            0,
        )
    };

    // Convert NTSTATUS to Result
    if status.is_err() {
        return Err(Error::from_hresult(HRESULT::from_nt(status.0)));
    }

    // Wrap the handle in Owned for automatic cleanup
    Ok(unsafe { Owned::new(key_handle) })
}
