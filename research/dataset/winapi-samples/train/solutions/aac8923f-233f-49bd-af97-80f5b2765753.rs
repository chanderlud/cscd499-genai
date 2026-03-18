use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::E_INVALIDARG;
use windows::Win32::Security::Cryptography::*;

struct HashHandle {
    handle: BCRYPT_HASH_HANDLE,
}

impl HashHandle {
    fn new() -> Result<Self> {
        Ok(Self {
            handle: BCRYPT_HASH_HANDLE::default(),
        })
    }
}

impl Drop for HashHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = BCryptDestroyHash(self.handle);
        }
    }
}

struct HkdfSha256 {
    algorithm: BCRYPT_ALG_HANDLE,
}

impl HkdfSha256 {
    fn new() -> Result<Self> {
        let mut algorithm = BCRYPT_ALG_HANDLE::default();

        unsafe {
            BCryptOpenAlgorithmProvider(
                &mut algorithm,
                BCRYPT_SHA256_ALGORITHM,
                None,
                BCRYPT_ALG_HANDLE_HMAC_FLAG,
            )
            .ok()?;
        }

        Ok(Self { algorithm })
    }

    fn extract(&self, salt: &[u8], ikm: &[u8]) -> Result<Vec<u8>> {
        // RFC 5869: If salt is empty, use HashLen zeros
        let effective_salt = if salt.is_empty() {
            vec![0u8; 32] // SHA-256 hash length
        } else {
            salt.to_vec()
        };

        self.hmac_sha256(&effective_salt, ikm)
    }

    fn expand(&self, prk: &[u8], info: &[u8], length: usize) -> Result<Vec<u8>> {
        const HASH_LEN: usize = 32; // SHA-256

        // RFC 5869: Check output length limit
        if length > 255 * HASH_LEN {
            return Err(Error::from_hresult(E_INVALIDARG));
        }

        let mut output = Vec::with_capacity(length);
        let mut t = Vec::new();
        let mut counter: u8 = 1;

        while output.len() < length {
            // T(i) = HMAC-Hash(PRK, T(i-1) || info || i)
            let mut data = Vec::new();
            data.extend_from_slice(&t);
            data.extend_from_slice(info);
            data.push(counter);

            t = self.hmac_sha256(prk, &data)?;
            output.extend_from_slice(&t);

            counter = counter
                .checked_add(1)
                .ok_or_else(|| Error::from_hresult(E_INVALIDARG))?;
        }

        output.truncate(length);
        Ok(output)
    }

    fn hmac_sha256(&self, key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
        let mut hash_handle = HashHandle::new()?;

        unsafe {
            // Create HMAC with key
            BCryptCreateHash(
                self.algorithm,
                &mut hash_handle.handle,
                None,
                if key.is_empty() { None } else { Some(key) },
                0,
            )
            .ok()?;

            // Hash the data
            if !data.is_empty() {
                BCryptHashData(hash_handle.handle, data, 0).ok()?;
            }

            // Get hash length
            let mut hash_length_bytes = [0u8; 4];
            let mut result_length = 0u32;
            BCryptGetProperty(
                self.algorithm.into(),
                BCRYPT_HASH_LENGTH,
                Some(&mut hash_length_bytes),
                &mut result_length,
                0,
            )
            .ok()?;

            let hash_length = u32::from_le_bytes(hash_length_bytes);

            // Finish hash and get result
            let mut hash_result = vec![0u8; hash_length as usize];
            BCryptFinishHash(hash_handle.handle, &mut hash_result, 0).ok()?;

            Ok(hash_result)
        }
    }
}

impl Drop for HkdfSha256 {
    fn drop(&mut self) {
        unsafe {
            let _ = BCryptCloseAlgorithmProvider(self.algorithm, 0);
        }
    }
}
