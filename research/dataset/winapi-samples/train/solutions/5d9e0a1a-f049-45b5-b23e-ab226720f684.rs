use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::ERROR_INVALID_FUNCTION;
use windows::Win32::Security::Cryptography::{
    BCryptCloseAlgorithmProvider, BCryptGenRandom, BCryptGetFipsAlgorithmMode,
    BCryptOpenAlgorithmProvider, BCRYPTGENRANDOM_FLAGS, BCRYPT_ALG_HANDLE,
    BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS, BCRYPT_RNG_ALGORITHM, BCRYPT_USE_SYSTEM_PREFERRED_RNG,
};

struct AlgorithmHandle(BCRYPT_ALG_HANDLE);

impl Drop for AlgorithmHandle {
    fn drop(&mut self) {
        // SAFETY: We own the handle and are closing it in drop
        unsafe {
            let _ = BCryptCloseAlgorithmProvider(self.0, 0);
        }
    }
}

pub fn generate_fips_random_bytes(byte_count: usize) -> Result<Vec<u8>> {
    // Check if FIPS mode is enabled
    let mut fips_enabled: u8 = 0;
    // SAFETY: BCryptGetFipsAlgorithmMode is safe to call with valid pointer
    unsafe {
        BCryptGetFipsAlgorithmMode(&mut fips_enabled).ok()?;
    }

    if fips_enabled == 0 {
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_INVALID_FUNCTION.0,
        )));
    }

    // Open FIPS-compliant RNG algorithm provider
    let mut algorithm_handle = BCRYPT_ALG_HANDLE::default();
    // SAFETY: BCryptOpenAlgorithmProvider is safe to call with valid parameters
    unsafe {
        BCryptOpenAlgorithmProvider(
            &mut algorithm_handle,
            BCRYPT_RNG_ALGORITHM,
            None,
            BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(0),
        )
        .ok()?;
    }

    let algorithm_guard = AlgorithmHandle(algorithm_handle);

    // Generate random bytes
    let mut buffer = vec![0u8; byte_count];
    // SAFETY: BCryptGenRandom is safe to call with valid handle and buffer
    unsafe {
        BCryptGenRandom(
            Some(algorithm_guard.0),
            buffer.as_mut_slice(),
            BCRYPTGENRANDOM_FLAGS(0),
        )
        .ok()?;
    }

    Ok(buffer)
}
