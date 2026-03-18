use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::core::Result;
use windows::Win32::Security::Cryptography::{
    BCryptEnumAlgorithms, BCryptFreeBuffer, BCryptGetFipsAlgorithmMode,
    BCRYPT_ALGORITHM_IDENTIFIER, BCRYPT_CIPHER_OPERATION,
};

const BCRYPT_FIPS_ALGORITHM_FLAG: u32 = 0x00000001;

fn get_fips_approved_symmetric_algorithms() -> Result<Vec<String>> {
    // First check if FIPS mode is enabled
    let mut fips_enabled = 0u8;
    // SAFETY: BCryptGetFipsAlgorithmMode writes to a BOOLEAN (u8) pointer
    let status = unsafe { BCryptGetFipsAlgorithmMode(&mut fips_enabled) };
    status.ok()?;

    if fips_enabled == 0 {
        // FIPS mode not enabled, return empty vector
        return Ok(Vec::new());
    }

    // Enumerate symmetric encryption algorithms
    let mut algorithm_count = 0u32;
    let mut algorithm_ptr = std::ptr::null_mut();

    // SAFETY: BCryptEnumAlgorithms allocates buffer for algorithm identifiers
    let status = unsafe {
        BCryptEnumAlgorithms(
            BCRYPT_CIPHER_OPERATION,
            &mut algorithm_count,
            &mut algorithm_ptr,
            0,
        )
    };

    // Convert NTSTATUS to Result
    status.ok()?;

    // Ensure buffer is freed even if we return early
    struct AlgorithmBufferGuard(*mut BCRYPT_ALGORITHM_IDENTIFIER);
    impl Drop for AlgorithmBufferGuard {
        fn drop(&mut self) {
            if !self.0.is_null() {
                // SAFETY: We're freeing a buffer allocated by BCryptEnumAlgorithms
                unsafe { BCryptFreeBuffer(self.0 as *const _) };
            }
        }
    }

    let _guard = AlgorithmBufferGuard(algorithm_ptr);

    if algorithm_ptr.is_null() || algorithm_count == 0 {
        return Ok(Vec::new());
    }

    // SAFETY: algorithm_ptr points to algorithm_count BCRYPT_ALGORITHM_IDENTIFIER structures
    let algorithms = unsafe { std::slice::from_raw_parts(algorithm_ptr, algorithm_count as usize) };

    let mut result = Vec::new();

    for algorithm in algorithms {
        // Check if algorithm is FIPS-approved
        if algorithm.dwFlags & BCRYPT_FIPS_ALGORITHM_FLAG != 0 {
            // Convert wide string to Rust String
            // SAFETY: pszName is a valid null-terminated wide string from Windows API
            let name = unsafe {
                let wide_name = algorithm.pszName;
                if wide_name.is_null() {
                    continue;
                }

                // Find the length of the null-terminated wide string
                let mut len = 0;
                while *wide_name.0.offset(len) != 0 {
                    len += 1;
                }

                // Convert to OsString then to String
                let wide_slice = std::slice::from_raw_parts(wide_name.0, len as usize);
                OsString::from_wide(wide_slice)
                    .to_string_lossy()
                    .into_owned()
            };

            result.push(name);
        }
    }

    Ok(result)
}
