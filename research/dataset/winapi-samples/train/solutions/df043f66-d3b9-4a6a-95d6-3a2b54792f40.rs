use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::NTSTATUS;
use windows::Win32::Security::Cryptography::{
    BCryptCloseAlgorithmProvider, BCryptGenRandom, BCryptOpenAlgorithmProvider,
    BCRYPTGENRANDOM_FLAGS, BCRYPT_ALG_HANDLE, BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS,
    BCRYPT_RNG_ALGORITHM,
};

fn generate_random_bytes(len: usize) -> Result<Vec<u8>> {
    let mut alg_handle = BCRYPT_ALG_HANDLE::default();

    // Open RNG algorithm provider
    let status = unsafe {
        BCryptOpenAlgorithmProvider(
            &mut alg_handle,
            BCRYPT_RNG_ALGORITHM,
            PCWSTR::null(),
            BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(0),
        )
    };

    if status != NTSTATUS(0) {
        return Err(Error::from_hresult(HRESULT::from_nt(status.0)));
    }

    // Ensure provider is closed even if generation fails
    let result = (|| {
        let mut buffer = vec![0u8; len];

        // Generate random bytes
        let status = unsafe {
            BCryptGenRandom(
                Some(alg_handle),
                buffer.as_mut_slice(),
                BCRYPTGENRANDOM_FLAGS(0),
            )
        };

        if status != NTSTATUS(0) {
            return Err(Error::from_hresult(HRESULT::from_nt(status.0)));
        }

        Ok(buffer)
    })();

    // Close algorithm provider
    let close_status = unsafe { BCryptCloseAlgorithmProvider(alg_handle, 0) };

    if close_status != NTSTATUS(0) {
        // If generation succeeded but close failed, return close error
        // If generation failed, return the original error
        if result.is_ok() {
            return Err(Error::from_hresult(HRESULT::from_nt(close_status.0)));
        }
    }

    result
}
