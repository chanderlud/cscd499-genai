use std::mem;
use windows::core::{Error, Owned, Result, HRESULT};
use windows::Win32::Security::Cryptography::{
    BCryptImportKeyPair, BCRYPT_ALG_HANDLE, BCRYPT_DSA_KEY_BLOB, BCRYPT_DSA_PUBLIC_BLOB,
    BCRYPT_DSA_PUBLIC_MAGIC, BCRYPT_KEY_HANDLE,
};

fn import_dsa_public_key(
    alg_handle: BCRYPT_ALG_HANDLE,
    p: &[u8],
    q: &[u8],
    g: &[u8],
    y: &[u8],
) -> Result<Owned<BCRYPT_KEY_HANDLE>> {
    // Validate q length (must be 20 bytes for DSA)
    if q.len() != 20 {
        return Err(Error::from_hresult(HRESULT::from_win32(0x80070057))); // E_INVALIDARG
    }

    // Validate p, g, y have same length
    if p.len() != g.len() || p.len() != y.len() {
        return Err(Error::from_hresult(HRESULT::from_win32(0x80070057)));
    }

    // Validate key size (must be 128, 256, or 384 bytes for 1024, 2048, 3072 bits)
    let key_size = p.len();
    if ![128, 256, 384].contains(&key_size) {
        return Err(Error::from_hresult(HRESULT::from_win32(0x80070057)));
    }

    // Convert from big-endian to little-endian
    let p_le: Vec<u8> = p.iter().rev().copied().collect();
    let q_le: Vec<u8> = q.iter().rev().copied().collect();
    let g_le: Vec<u8> = g.iter().rev().copied().collect();
    let y_le: Vec<u8> = y.iter().rev().copied().collect();

    // Build BCRYPT_DSA_KEY_BLOB header
    let header = BCRYPT_DSA_KEY_BLOB {
        dwMagic: BCRYPT_DSA_PUBLIC_MAGIC,
        cbKey: key_size as u32,
        Count: [0; 4],
        Seed: [0; 20],
        q: {
            let mut q_arr = [0u8; 20];
            q_arr.copy_from_slice(&q_le);
            q_arr
        },
    };

    // Calculate total blob size
    let header_size = mem::size_of::<BCRYPT_DSA_KEY_BLOB>();
    let total_size = header_size + (key_size * 3); // p + g + y

    // Build the complete blob
    let mut blob = Vec::with_capacity(total_size);

    // SAFETY: BCRYPT_DSA_KEY_BLOB is a plain-old-data struct with repr(C)
    unsafe {
        let header_bytes =
            std::slice::from_raw_parts(&header as *const _ as *const u8, header_size);
        blob.extend_from_slice(header_bytes);
    }

    blob.extend_from_slice(&p_le);
    blob.extend_from_slice(&g_le);
    blob.extend_from_slice(&y_le);

    // Import the key
    let mut key_handle = BCRYPT_KEY_HANDLE::default();

    // SAFETY: BCryptImportKeyPair is safe to call with valid parameters
    // Use .ok()? to convert NTSTATUS to Result
    unsafe {
        BCryptImportKeyPair(
            alg_handle,
            None,
            BCRYPT_DSA_PUBLIC_BLOB,
            &mut key_handle,
            &blob,
            0,
        )
    }
    .ok()?; // This handles the NTSTATUS -> Result conversion

    // SAFETY: key_handle is valid after successful BCryptImportKeyPair
    Ok(unsafe { Owned::new(key_handle) })
}
