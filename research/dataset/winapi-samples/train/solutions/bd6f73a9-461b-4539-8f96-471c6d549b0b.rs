use std::cell::OnceCell;
use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::STATUS_SUCCESS;
use windows::Win32::Security::Cryptography::*;

thread_local! {
    static AES_GCM_ALG: OnceCell<BCRYPT_ALG_HANDLE> = const { OnceCell::new() };
}

struct KeyGuard(BCRYPT_KEY_HANDLE);

impl Drop for KeyGuard {
    fn drop(&mut self) {
        // SAFETY: We own this key handle and must destroy it when done
        unsafe {
            let _ = BCryptDestroyKey(self.0);
        }
    }
}

fn get_aes_gcm_alg() -> Result<BCRYPT_ALG_HANDLE> {
    AES_GCM_ALG.with(|cell| {
        let handle = cell.get_or_init(|| {
            let mut alg_handle = BCRYPT_ALG_HANDLE::default();
            // SAFETY: We're calling BCryptOpenAlgorithmProvider with valid parameters
            let status = unsafe {
                BCryptOpenAlgorithmProvider(
                    &mut alg_handle,
                    BCRYPT_AES_ALGORITHM,
                    None,
                    BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(0),
                )
            };
            if status != STATUS_SUCCESS {
                return BCRYPT_ALG_HANDLE::default();
            }

            // Set the chaining mode to GCM
            // Convert PCWSTR to &[u8] for BCryptSetProperty
            let mode_wide = BCRYPT_CHAIN_MODE_GCM;
            let mode_bytes = unsafe {
                // SAFETY: mode_wide is a valid null-terminated wide string constant
                let mut len = 0;
                while *mode_wide.0.offset(len) != 0 {
                    len += 1;
                }
                // Include null terminator, each wide char is 2 bytes
                std::slice::from_raw_parts(mode_wide.0 as *const u8, (len as usize + 1) * 2)
            };

            let status = unsafe {
                BCryptSetProperty(
                    BCRYPT_HANDLE(alg_handle.0), // Convert BCRYPT_ALG_HANDLE to BCRYPT_HANDLE
                    BCRYPT_CHAINING_MODE,
                    mode_bytes,
                    0,
                )
            };
            if status != STATUS_SUCCESS {
                return BCRYPT_ALG_HANDLE::default();
            }

            alg_handle
        });

        if handle.0.is_null() {
            return Err(Error::from_hresult(HRESULT::from_win32(0x80070005)));
        }
        Ok(*handle)
    })
}

pub fn encrypt_aes_gcm(
    key: &[u8; 32],
    nonce: &[u8; 12],
    plaintext: &[u8],
    aad: &[u8],
) -> Result<(Vec<u8>, [u8; 16])> {
    let alg_handle = get_aes_gcm_alg()?;

    let mut key_handle = BCRYPT_KEY_HANDLE::default();
    // SAFETY: We're calling BCryptGenerateSymmetricKey with valid parameters
    let status = unsafe { BCryptGenerateSymmetricKey(alg_handle, &mut key_handle, None, key, 0) };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(status.to_hresult()));
    }
    let _key_guard = KeyGuard(key_handle);

    let mut auth_info = BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO {
        cbSize: std::mem::size_of::<BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO>() as u32,
        dwInfoVersion: BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO_VERSION,
        pbNonce: nonce.as_ptr() as *mut u8,
        cbNonce: nonce.len() as u32,
        pbAuthData: aad.as_ptr() as *mut u8,
        cbAuthData: aad.len() as u32,
        pbTag: std::ptr::null_mut(),
        cbTag: 0,
        ..Default::default()
    };

    let mut tag = [0u8; 16];
    auth_info.pbTag = tag.as_mut_ptr();
    auth_info.cbTag = tag.len() as u32;

    let mut ciphertext = vec![0u8; plaintext.len()];
    let mut result_len = 0u32;

    // SAFETY: We're calling BCryptEncrypt with valid parameters
    let status = unsafe {
        BCryptEncrypt(
            key_handle,
            Some(plaintext),
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

    Ok((ciphertext, tag))
}
