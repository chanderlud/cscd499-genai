use std::cell::RefCell;
use windows::core::{w, Error, Result};
use windows::Win32::Foundation::STATUS_SUCCESS;
use windows::Win32::Security::Cryptography::*;

// Thread-local BCrypt algorithm provider for AES-GCM
thread_local! {
    static AES_GCM_ALG: RefCell<Option<BCRYPT_ALG_HANDLE>> = const { RefCell::new(None) };
}

fn get_aes_gcm_provider() -> Result<BCRYPT_ALG_HANDLE> {
    AES_GCM_ALG.with(|cell| {
        let mut borrow = cell.borrow_mut();
        if let Some(alg_handle) = *borrow {
            return Ok(alg_handle);
        }

        // Open algorithm provider for AES
        let mut alg_handle = BCRYPT_ALG_HANDLE::default();
        let status = unsafe {
            BCryptOpenAlgorithmProvider(
                &mut alg_handle,
                w!("AES"),
                None,
                BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(0),
            )
        };

        if status != STATUS_SUCCESS {
            return Err(Error::from_hresult(status.to_hresult()));
        }

        // Set chain mode to GCM
        // BCRYPT_CHAIN_MODE_GCM is already a PCWSTR, so we can use it directly
        let chain_mode_wide = BCRYPT_CHAIN_MODE_GCM;
        let chain_mode_bytes = unsafe {
            // Calculate length of null-terminated wide string
            let mut len = 0;
            while *chain_mode_wide.0.offset(len) != 0 {
                len += 1;
            }
            std::slice::from_raw_parts(
                chain_mode_wide.0 as *const u8,
                (len as usize + 1) * 2, // Include null terminator
            )
        };

        // Convert BCRYPT_ALG_HANDLE to BCRYPT_HANDLE for BCryptSetProperty
        let handle = BCRYPT_HANDLE(alg_handle.0);
        let status = unsafe { BCryptSetProperty(handle, w!("ChainingMode"), chain_mode_bytes, 0) };

        if status != STATUS_SUCCESS {
            let _ = unsafe { BCryptCloseAlgorithmProvider(alg_handle, 0) };
            return Err(Error::from_hresult(status.to_hresult()));
        }

        *borrow = Some(alg_handle);
        Ok(alg_handle)
    })
}

// Simple guard for BCrypt key handle
struct KeyGuard(BCRYPT_KEY_HANDLE);

impl Drop for KeyGuard {
    fn drop(&mut self) {
        let _ = unsafe { BCryptDestroyKey(self.0) };
    }
}

pub fn encrypt_aes_gcm_iter<I, J>(
    key: &[u8; 32],
    nonce: &[u8; 12],
    plaintext: I,
    aad: J,
) -> Result<(Vec<u8>, [u8; 16])>
where
    I: IntoIterator<Item = u8>,
    J: IntoIterator<Item = u8>,
{
    // Collect plaintext and AAD from iterators
    let plaintext_vec: Vec<u8> = plaintext.into_iter().collect();
    let aad_vec: Vec<u8> = aad.into_iter().collect();

    // Get AES-GCM provider
    let alg_handle = get_aes_gcm_provider()?;

    // Generate symmetric key from raw key bytes
    let mut key_handle = BCRYPT_KEY_HANDLE::default();
    let status = unsafe { BCryptGenerateSymmetricKey(alg_handle, &mut key_handle, None, key, 0) };

    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(status.to_hresult()));
    }

    // Ensure key is destroyed when we're done
    let _key_guard = KeyGuard(key_handle);

    // Prepare buffers for encryption
    let mut ciphertext = vec![0u8; plaintext_vec.len()];
    let mut tag = [0u8; 16];
    let mut result_len = 0u32;

    // Set up authenticated cipher mode info structure
    let auth_info = BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO {
        cbSize: std::mem::size_of::<BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO>() as u32,
        dwInfoVersion: BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO_VERSION,
        pbNonce: nonce.as_ptr() as *mut u8,
        cbNonce: nonce.len() as u32,
        pbAuthData: if aad_vec.is_empty() {
            std::ptr::null_mut()
        } else {
            aad_vec.as_ptr() as *mut u8
        },
        cbAuthData: aad_vec.len() as u32,
        pbTag: tag.as_mut_ptr(),
        cbTag: tag.len() as u32,
        pbMacContext: std::ptr::null_mut(),
        cbMacContext: 0,
        cbAAD: aad_vec.len() as u32,
        cbData: plaintext_vec.len() as u64,
        dwFlags: 0,
    };

    // Perform encryption
    let status = unsafe {
        BCryptEncrypt(
            key_handle,
            Some(plaintext_vec.as_slice()),
            Some(&auth_info as *const _ as *const std::ffi::c_void),
            None,
            Some(ciphertext.as_mut_slice()),
            &mut result_len,
            BCRYPT_FLAGS(0),
        )
    };

    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(status.to_hresult()));
    }

    // Resize ciphertext to actual size (should be same as plaintext for GCM)
    ciphertext.truncate(result_len as usize);

    Ok((ciphertext, tag))
}
