use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
#[allow(unused_imports)]
use windows::core::{Error, Result};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

pub fn shell_stream_sha256(path: &Path) -> Result<[u8; 32]> {
    use windows::Win32::Foundation::S_OK;
    use windows::Win32::Security::Cryptography::{
        BCryptCloseAlgorithmProvider, BCryptCreateHash, BCryptDestroyHash, BCryptFinishHash,
        BCryptHashData, BCryptOpenAlgorithmProvider, BCRYPT_ALG_HANDLE, BCRYPT_HASH_HANDLE,
        BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS, BCRYPT_SHA256_ALGORITHM,
    };
    use windows::Win32::System::Com::{
        CoInitializeEx, IStream, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
    };
    use windows::Win32::UI::Shell::{BHID_Stream, IShellItem, SHCreateItemFromParsingName};

    // Initialize COM for this thread
    unsafe {
        let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE);
        // S_OK and S_FALSE are both success codes
        if hr.is_err() {
            return Err(Error::from_hresult(hr));
        }
    }

    // Create shell item from path
    let shell_item: IShellItem = unsafe {
        let path_wide = wide_null(path.as_os_str());
        SHCreateItemFromParsingName(windows::core::PCWSTR(path_wide.as_ptr()), None)?
    };

    // Bind to stream handler
    let stream: IStream = unsafe { shell_item.BindToHandler(None, &BHID_Stream)? };

    // Initialize BCrypt for SHA-256
    let mut alg_handle = BCRYPT_ALG_HANDLE::default();
    let mut hash_handle = BCRYPT_HASH_HANDLE::default();

    unsafe {
        let status = BCryptOpenAlgorithmProvider(
            &mut alg_handle,
            BCRYPT_SHA256_ALGORITHM,
            None,
            BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(0),
        );
        if status.is_err() {
            return Err(Error::from_hresult(status.into()));
        }
    }

    // Create hash object
    unsafe {
        let status = BCryptCreateHash(alg_handle, &mut hash_handle, None, None, 0);
        if status.is_err() {
            let _ = BCryptCloseAlgorithmProvider(alg_handle, 0);
            return Err(Error::from_hresult(status.into()));
        }
    }

    // Read stream and update hash
    let mut buffer = [0u8; 4096];
    loop {
        let mut bytes_read = 0u32;
        unsafe {
            let hr = stream.Read(
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u32,
                Some(&mut bytes_read),
            );

            // Only treat actual errors as failures
            // S_OK (0) and S_FALSE (1) are both success codes
            if hr.is_err() {
                let _ = BCryptDestroyHash(hash_handle);
                let _ = BCryptCloseAlgorithmProvider(alg_handle, 0);
                return Err(Error::from_hresult(hr));
            }
        }

        if bytes_read == 0 {
            break;
        }

        unsafe {
            let status = BCryptHashData(hash_handle, &buffer[..bytes_read as usize], 0);
            if status.is_err() {
                let _ = BCryptDestroyHash(hash_handle);
                let _ = BCryptCloseAlgorithmProvider(alg_handle, 0);
                return Err(Error::from_hresult(status.into()));
            }
        }
    }

    // Finalize hash and get result
    let mut digest = [0u8; 32];
    unsafe {
        let status = BCryptFinishHash(hash_handle, &mut digest, 0);
        let _ = BCryptDestroyHash(hash_handle);
        let _ = BCryptCloseAlgorithmProvider(alg_handle, 0);

        if status.is_err() {
            return Err(Error::from_hresult(status.into()));
        }
    }

    Ok(digest)
}
