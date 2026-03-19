use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::E_INVALIDARG;
use windows::Win32::Security::Cryptography::*;

pub fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    s.encode_wide().chain(std::iter::once(0)).collect()
}

pub fn open_aes_provider() -> Result<BCRYPT_ALG_HANDLE> {
    let mut handle = BCRYPT_ALG_HANDLE::default();
    let status = unsafe {
        BCryptOpenAlgorithmProvider(
            &mut handle,
            PCWSTR(wide_null("AES".as_ref()).as_ptr()),
            PCWSTR::null(),
            BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(0),
        )
    };
    if status.is_err() {
        return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
    }
    Ok(handle)
}

pub fn set_gcm_mode(alg_handle: BCRYPT_ALG_HANDLE) -> Result<()> {
    let mode = wide_null("ChainingModeGCM".as_ref());
    let mode_bytes: Vec<u8> = mode.iter().flat_map(|&c| c.to_le_bytes()).collect();
    let status = unsafe {
        BCryptSetProperty(
            alg_handle.into(),
            PCWSTR(wide_null("ChainingMode".as_ref()).as_ptr()),
            &mode_bytes,
            0,
        )
    };
    if status.is_err() {
        return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
    }
    Ok(())
}

pub fn create_key_handle(alg_handle: BCRYPT_ALG_HANDLE, key: &[u8]) -> Result<BCRYPT_KEY_HANDLE> {
    let mut key_handle = BCRYPT_KEY_HANDLE::default();
    let status = unsafe { BCryptGenerateSymmetricKey(alg_handle, &mut key_handle, None, key, 0) };
    if status.is_err() {
        return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
    }
    Ok(key_handle)
}

pub fn aes_gcm_encrypt(
    key: &[u8],
    nonce: &[u8],
    plaintext: &[u8],
    aad: Option<&[u8]>,
) -> Result<(Vec<u8>, [u8; 16])> {
    if nonce.len() != 12 {
        return Err(Error::from_hresult(E_INVALIDARG));
    }
    if key.len() != 16 && key.len() != 24 && key.len() != 32 {
        return Err(Error::from_hresult(E_INVALIDARG));
    }
    if plaintext.len() > (u32::MAX - 2) as usize {
        return Err(Error::from_hresult(E_INVALIDARG));
    }

    let alg_handle = open_aes_provider()?;
    set_gcm_mode(alg_handle)?;
    let key_handle = create_key_handle(alg_handle, key)?;

    let mut auth_info = BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO {
        cbSize: std::mem::size_of::<BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO>() as u32,
        dwInfoVersion: BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO_VERSION,
        pbNonce: nonce.as_ptr() as *mut u8,
        cbNonce: nonce.len() as u32,
        cbTag: 16,
        ..Default::default()
    };

    if let Some(aad_data) = aad {
        auth_info.pbAuthData = aad_data.as_ptr() as *mut u8;
        auth_info.cbAuthData = aad_data.len() as u32;
    }

    let mut ciphertext = vec![0u8; plaintext.len()];
    let mut result_len = 0u32;
    let mut tag = [0u8; 16];
    auth_info.pbTag = tag.as_mut_ptr();

    let status = unsafe {
        BCryptEncrypt(
            key_handle,
            Some(plaintext),
            Some(&auth_info as *const _ as *const _),
            None,
            Some(ciphertext.as_mut_slice()),
            &mut result_len,
            BCRYPT_FLAGS(0),
        )
    };

    let _ = unsafe { BCryptDestroyKey(key_handle) };
    let _ = unsafe { BCryptCloseAlgorithmProvider(alg_handle, 0) };

    if status.is_err() {
        return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
    }

    ciphertext.truncate(result_len as usize);
    Ok((ciphertext, tag))
}

pub fn aes_gcm_decrypt(
    key: &[u8],
    nonce: &[u8],
    ciphertext: &[u8],
    tag: &[u8; 16],
    aad: Option<&[u8]>,
) -> Result<Vec<u8>> {
    if nonce.len() != 12 {
        return Err(Error::from_hresult(E_INVALIDARG));
    }
    if key.len() != 16 && key.len() != 24 && key.len() != 32 {
        return Err(Error::from_hresult(E_INVALIDARG));
    }
    if ciphertext.len() > (u32::MAX - 2) as usize {
        return Err(Error::from_hresult(E_INVALIDARG));
    }

    let alg_handle = open_aes_provider()?;
    set_gcm_mode(alg_handle)?;
    let key_handle = create_key_handle(alg_handle, key)?;

    let mut auth_info = BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO {
        cbSize: std::mem::size_of::<BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO>() as u32,
        dwInfoVersion: BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO_VERSION,
        pbNonce: nonce.as_ptr() as *mut u8,
        cbNonce: nonce.len() as u32,
        cbTag: tag.len() as u32,
        pbTag: tag.as_ptr() as *mut u8,
        ..Default::default()
    };

    if let Some(aad_data) = aad {
        auth_info.pbAuthData = aad_data.as_ptr() as *mut u8;
        auth_info.cbAuthData = aad_data.len() as u32;
    }

    let mut plaintext = vec![0u8; ciphertext.len()];
    let mut result_len = 0u32;

    let status = unsafe {
        BCryptDecrypt(
            key_handle,
            Some(ciphertext),
            Some(&auth_info as *const _ as *const _),
            None,
            Some(plaintext.as_mut_slice()),
            &mut result_len,
            BCRYPT_FLAGS(0),
        )
    };

    let _ = unsafe { BCryptDestroyKey(key_handle) };
    let _ = unsafe { BCryptCloseAlgorithmProvider(alg_handle, 0) };

    if status.is_err() {
        if status.0 == -1073700864 {
            return Err(Error::from_hresult(E_INVALIDARG));
        }
        return Err(Error::from_hresult(HRESULT::from_win32(status.0 as u32)));
    }

    plaintext.truncate(result_len as usize);
    Ok(plaintext)
}
