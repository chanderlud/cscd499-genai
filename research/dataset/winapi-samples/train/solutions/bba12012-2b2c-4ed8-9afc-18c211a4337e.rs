use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::STATUS_SUCCESS;
use windows::Win32::Security::Cryptography::{
    BCryptCloseAlgorithmProvider, BCryptDestroyKey, BCryptEncrypt, BCryptGenerateSymmetricKey,
    BCryptOpenAlgorithmProvider, BCryptSetProperty, BCRYPT_AES_ALGORITHM, BCRYPT_ALG_HANDLE,
    BCRYPT_BLOCK_PADDING, BCRYPT_CHAIN_MODE_ECB, BCRYPT_KEY_HANDLE,
    BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS,
};

fn wrap_key(wrapping_key: &[u8], key_to_wrap: &[u8]) -> Result<Vec<u8>> {
    // Validate key sizes
    if ![16, 24, 32].contains(&wrapping_key.len()) {
        return Err(Error::from_hresult(HRESULT::from_win32(0x80070057))); // E_INVALIDARG
    }
    if wrapping_key.len() != key_to_wrap.len() {
        return Err(Error::from_hresult(HRESULT::from_win32(0x80070057))); // E_INVALIDARG
    }

    // Default IV for AES Key Wrap (RFC 3394)
    const DEFAULT_IV: [u8; 8] = [0xA6, 0xA6, 0xA6, 0xA6, 0xA6, 0xA6, 0xA6, 0xA6];

    // Prepare input: IV || key_to_wrap
    let mut input_data = Vec::with_capacity(DEFAULT_IV.len() + key_to_wrap.len());
    input_data.extend_from_slice(&DEFAULT_IV);
    input_data.extend_from_slice(key_to_wrap);

    unsafe {
        // Open algorithm handle
        let mut alg_handle = BCRYPT_ALG_HANDLE::default();
        let status = BCryptOpenAlgorithmProvider(
            &mut alg_handle,
            BCRYPT_AES_ALGORITHM,
            None,
            BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(0),
        );
        if status != STATUS_SUCCESS {
            return Err(Error::from_hresult(HRESULT::from_nt(status.0)));
        }

        // Set chain mode to ECB
        let chain_mode = BCRYPT_CHAIN_MODE_ECB;
        let status = BCryptSetProperty(alg_handle.into(), chain_mode, &[], 0);
        if status != STATUS_SUCCESS {
            let _ = BCryptCloseAlgorithmProvider(alg_handle, 0);
            return Err(Error::from_hresult(HRESULT::from_nt(status.0)));
        }

        // Generate symmetric key from wrapping key
        let mut key_handle = BCRYPT_KEY_HANDLE::default();
        let status = BCryptGenerateSymmetricKey(alg_handle, &mut key_handle, None, wrapping_key, 0);
        if status != STATUS_SUCCESS {
            let _ = BCryptCloseAlgorithmProvider(alg_handle, 0);
            return Err(Error::from_hresult(HRESULT::from_nt(status.0)));
        }

        // Calculate output size (input_data.len() + 8 bytes for padding)
        let output_size = input_data.len() + 8;
        let mut output_data = vec![0u8; output_size];
        let mut bytes_written = 0u32;

        // Perform encryption (AES Key Wrap)
        let status = BCryptEncrypt(
            key_handle,
            Some(&input_data),
            None,
            None,
            Some(&mut output_data),
            &mut bytes_written,
            BCRYPT_BLOCK_PADDING,
        );

        // Clean up handles
        let _ = BCryptDestroyKey(key_handle);
        let _ = BCryptCloseAlgorithmProvider(alg_handle, 0);

        if status != STATUS_SUCCESS {
            return Err(Error::from_hresult(HRESULT::from_nt(status.0)));
        }

        // Resize to actual bytes written
        output_data.truncate(bytes_written as usize);
        Ok(output_data)
    }
}
