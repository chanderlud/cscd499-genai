use std::mem;
use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::STATUS_SUCCESS;
use windows::Win32::Security::Cryptography::{
    BCryptCloseAlgorithmProvider, BCryptDecrypt, BCryptDestroyKey, BCryptEncrypt, BCryptGenRandom,
    BCryptGenerateSymmetricKey, BCryptOpenAlgorithmProvider, BCryptSetProperty,
    BCRYPTGENRANDOM_FLAGS, BCRYPT_AES_ALGORITHM, BCRYPT_ALG_HANDLE,
    BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO, BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO_VERSION,
    BCRYPT_CHAINING_MODE, BCRYPT_CHAIN_MODE_GCM, BCRYPT_FLAGS, BCRYPT_KEY_HANDLE,
    BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS,
};

struct AesGcmCipher {
    alg_handle: BCRYPT_ALG_HANDLE,
    key_handle: BCRYPT_KEY_HANDLE,
}

impl AesGcmCipher {
    fn new(key: &[u8]) -> Result<Self> {
        if key.len() != 32 {
            return Err(Error::from_hresult(HRESULT::from_win32(0x80070057))); // E_INVALIDARG
        }

        let mut alg_handle = BCRYPT_ALG_HANDLE::default();
        let mut key_handle = BCRYPT_KEY_HANDLE::default();

        unsafe {
            // Open algorithm provider
            let status = BCryptOpenAlgorithmProvider(
                &mut alg_handle,
                BCRYPT_AES_ALGORITHM,
                None,
                BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS::default(),
            );
            if status != STATUS_SUCCESS {
                return Err(Error::from_hresult(status.to_hresult()));
            }

            // Set chaining mode to GCM
            let chaining_mode_wide = BCRYPT_CHAIN_MODE_GCM.as_wide();
            let chaining_mode_bytes = std::slice::from_raw_parts(
                chaining_mode_wide.as_ptr() as *const u8,
                chaining_mode_wide.len() * 2,
            );
            let status = BCryptSetProperty(
                alg_handle.into(),
                BCRYPT_CHAINING_MODE,
                chaining_mode_bytes,
                0,
            );
            if status != STATUS_SUCCESS {
                let _ = BCryptCloseAlgorithmProvider(alg_handle, 0);
                return Err(Error::from_hresult(status.to_hresult()));
            }

            // Generate symmetric key
            let status = BCryptGenerateSymmetricKey(alg_handle, &mut key_handle, None, key, 0);
            if status != STATUS_SUCCESS {
                let _ = BCryptCloseAlgorithmProvider(alg_handle, 0);
                return Err(Error::from_hresult(status.to_hresult()));
            }
        }

        Ok(Self {
            alg_handle,
            key_handle,
        })
    }

    fn encrypt(
        &self,
        plaintext: &[u8],
        associated_data: &[u8],
    ) -> Result<(Vec<u8>, [u8; 16], [u8; 12])> {
        let mut nonce = [0u8; 12];

        // Generate random nonce using BCrypt
        unsafe {
            let status = BCryptGenRandom(
                None, // Use system-preferred RNG
                &mut nonce,
                BCRYPTGENRANDOM_FLAGS(0), // Correct flag type
            );
            if status != STATUS_SUCCESS {
                return Err(Error::from_hresult(status.to_hresult()));
            }
        }

        let mut tag = [0u8; 16];
        let mut ciphertext = vec![0u8; plaintext.len()];

        let auth_info = BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO {
            cbSize: mem::size_of::<BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO>() as u32,
            dwInfoVersion: BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO_VERSION,
            pbNonce: nonce.as_ptr() as *mut u8,
            cbNonce: nonce.len() as u32,
            pbAuthData: if associated_data.is_empty() {
                std::ptr::null_mut()
            } else {
                associated_data.as_ptr() as *mut u8
            },
            cbAuthData: associated_data.len() as u32,
            pbTag: tag.as_mut_ptr(),
            cbTag: tag.len() as u32,
            pbMacContext: std::ptr::null_mut(),
            cbMacContext: 0,
            cbAAD: 0,
            cbData: 0,
            dwFlags: 0,
        };

        let mut result_len = 0u32;

        unsafe {
            let status = BCryptEncrypt(
                self.key_handle,
                Some(plaintext),
                Some(&auth_info as *const _ as *const std::ffi::c_void),
                None,
                Some(ciphertext.as_mut_slice()),
                &mut result_len,
                BCRYPT_FLAGS(0),
            );
            if status != STATUS_SUCCESS {
                return Err(Error::from_hresult(status.to_hresult()));
            }
        }

        ciphertext.truncate(result_len as usize);
        Ok((ciphertext, tag, nonce))
    }

    fn decrypt(
        &self,
        ciphertext: &[u8],
        associated_data: &[u8],
        tag: &[u8; 16],
        nonce: &[u8; 12],
    ) -> Result<Vec<u8>> {
        let mut plaintext = vec![0u8; ciphertext.len() + 16]; // Extra space for padding

        let auth_info = BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO {
            cbSize: mem::size_of::<BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO>() as u32,
            dwInfoVersion: BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO_VERSION,
            pbNonce: nonce.as_ptr() as *mut u8,
            cbNonce: nonce.len() as u32,
            pbAuthData: if associated_data.is_empty() {
                std::ptr::null_mut()
            } else {
                associated_data.as_ptr() as *mut u8
            },
            cbAuthData: associated_data.len() as u32,
            pbTag: tag.as_ptr() as *mut u8,
            cbTag: tag.len() as u32,
            pbMacContext: std::ptr::null_mut(),
            cbMacContext: 0,
            cbAAD: 0,
            cbData: 0,
            dwFlags: 0,
        };

        let mut result_len = 0u32;

        unsafe {
            let status = BCryptDecrypt(
                self.key_handle,
                Some(ciphertext),
                Some(&auth_info as *const _ as *const std::ffi::c_void),
                None,
                Some(plaintext.as_mut_slice()),
                &mut result_len,
                BCRYPT_FLAGS(0),
            );
            if status != STATUS_SUCCESS {
                return Err(Error::from_hresult(status.to_hresult()));
            }
        }

        plaintext.truncate(result_len as usize);
        Ok(plaintext)
    }
}

impl Drop for AesGcmCipher {
    fn drop(&mut self) {
        unsafe {
            if !self.key_handle.is_invalid() {
                let _ = BCryptDestroyKey(self.key_handle);
            }
            if !self.alg_handle.is_invalid() {
                let _ = BCryptCloseAlgorithmProvider(self.alg_handle, 0);
            }
        }
    }
}

fn main() -> Result<()> {
    let key = [1u8; 32]; // Example 32-byte key
    let cipher = AesGcmCipher::new(&key)?;

    let plaintext = b"Hello, World!";
    let associated_data = b"additional data";

    let (ciphertext, tag, nonce) = cipher.encrypt(plaintext, associated_data)?;
    let decrypted = cipher.decrypt(&ciphertext, associated_data, &tag, &nonce)?;

    assert_eq!(decrypted, plaintext);
    Ok(())
}
