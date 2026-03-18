use std::mem::MaybeUninit;
use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::STATUS_SUCCESS;
use windows::Win32::Security::Cryptography::{
    BCryptCloseAlgorithmProvider, BCryptDestroyKey, BCryptEncrypt, BCryptGenerateSymmetricKey,
    BCryptOpenAlgorithmProvider, BCryptSetProperty, BCRYPT_AES_ALGORITHM, BCRYPT_ALG_HANDLE,
    BCRYPT_BLOCK_PADDING, BCRYPT_CHAINING_MODE, BCRYPT_CHAIN_MODE_CBC, BCRYPT_HANDLE,
    BCRYPT_KEY_HANDLE, BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS,
};

struct AlgorithmProvider(BCRYPT_ALG_HANDLE);
impl Drop for AlgorithmProvider {
    fn drop(&mut self) {
        unsafe { BCryptCloseAlgorithmProvider(self.0, 0).ok().unwrap() };
    }
}

struct SymmetricKey(BCRYPT_KEY_HANDLE);
impl Drop for SymmetricKey {
    fn drop(&mut self) {
        unsafe { BCryptDestroyKey(self.0).ok().unwrap() };
    }
}

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

fn aes_cbc_encrypt(key: &[u8; 16], iv: &[u8; 16], plaintext: &[u8]) -> Result<Vec<u8>> {
    // Open algorithm provider
    let mut alg_handle = BCRYPT_ALG_HANDLE::default();
    let status = unsafe {
        BCryptOpenAlgorithmProvider(
            &mut alg_handle,
            BCRYPT_AES_ALGORITHM,
            PCWSTR::null(),
            BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(0),
        )
    };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(status.to_hresult()));
    }
    let alg = AlgorithmProvider(alg_handle);

    // Set chaining mode to CBC
    let chaining_mode = BCRYPT_CHAIN_MODE_CBC;
    let status = unsafe {
        BCryptSetProperty(
            BCRYPT_HANDLE(alg.0 .0),
            BCRYPT_CHAINING_MODE,
            unsafe {
                std::slice::from_raw_parts(
                    chaining_mode.0 as *const u8,
                    std::mem::size_of::<PCWSTR>(),
                )
            },
            0,
        )
    };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(status.to_hresult()));
    }

    // Generate symmetric key
    let mut key_handle = BCRYPT_KEY_HANDLE::default();
    let status = unsafe { BCryptGenerateSymmetricKey(alg.0, &mut key_handle, None, key, 0) };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(status.to_hresult()));
    }
    let key = SymmetricKey(key_handle);

    // Determine ciphertext size
    let mut ciphertext_len = MaybeUninit::<u32>::uninit();
    let mut iv_copy = *iv;
    let status = unsafe {
        BCryptEncrypt(
            key.0,
            Some(plaintext),
            None,
            Some(&mut iv_copy),
            None,
            ciphertext_len.as_mut_ptr(),
            BCRYPT_BLOCK_PADDING,
        )
    };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(status.to_hresult()));
    }
    let ciphertext_len = unsafe { ciphertext_len.assume_init() } as usize;

    // Perform encryption
    let mut ciphertext = vec![0u8; ciphertext_len];
    let mut iv_copy = *iv;
    let mut actual_len = MaybeUninit::<u32>::uninit();
    let status = unsafe {
        BCryptEncrypt(
            key.0,
            Some(plaintext),
            None,
            Some(&mut iv_copy),
            Some(&mut ciphertext),
            actual_len.as_mut_ptr(),
            BCRYPT_BLOCK_PADDING,
        )
    };
    if status != STATUS_SUCCESS {
        return Err(Error::from_hresult(status.to_hresult()));
    }
    let actual_len = unsafe { actual_len.assume_init() } as usize;
    ciphertext.truncate(actual_len);

    Ok(ciphertext)
}
