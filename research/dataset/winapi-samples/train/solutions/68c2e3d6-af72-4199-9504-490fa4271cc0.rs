use windows::core::Result;
use windows::Win32::Security::Cryptography::*;

fn remove_leading_zeros(bytes: &[u8]) -> &[u8] {
    let mut i = 0;
    while i < bytes.len() && bytes[i] == 0 {
        i += 1;
    }
    if i == bytes.len() {
        &bytes[bytes.len() - 1..]
    } else {
        &bytes[i..]
    }
}

fn build_rsa_private_blob(
    modulus: &[u8],
    public_exponent: &[u8],
    prime1: &[u8],
    prime2: &[u8],
) -> Vec<u8> {
    let modulus_be = remove_leading_zeros(modulus);
    let public_exponent_be = remove_leading_zeros(public_exponent);
    let prime1_be = remove_leading_zeros(prime1);
    let prime2_be = remove_leading_zeros(prime2);

    let modulus_le: Vec<u8> = modulus_be.iter().rev().cloned().collect();
    let public_exponent_le: Vec<u8> = public_exponent_be.iter().rev().cloned().collect();
    let prime1_le: Vec<u8> = prime1_be.iter().rev().cloned().collect();
    let prime2_le: Vec<u8> = prime2_be.iter().rev().cloned().collect();

    let bit_length = (modulus_be.len() * 8) as u32;

    let header = BCRYPT_RSAKEY_BLOB {
        Magic: BCRYPT_RSAPRIVATE_MAGIC,
        BitLength: bit_length,
        cbPublicExp: public_exponent_le.len() as u32,
        cbModulus: modulus_le.len() as u32,
        cbPrime1: prime1_le.len() as u32,
        cbPrime2: prime2_le.len() as u32,
    };

    let mut blob = Vec::new();
    blob.extend_from_slice(&header.Magic.0.to_ne_bytes());
    blob.extend_from_slice(&header.BitLength.to_ne_bytes());
    blob.extend_from_slice(&header.cbPublicExp.to_ne_bytes());
    blob.extend_from_slice(&header.cbModulus.to_ne_bytes());
    blob.extend_from_slice(&header.cbPrime1.to_ne_bytes());
    blob.extend_from_slice(&header.cbPrime2.to_ne_bytes());
    blob.extend_from_slice(&public_exponent_le);
    blob.extend_from_slice(&modulus_le);
    blob.extend_from_slice(&prime1_le);
    blob.extend_from_slice(&prime2_le);

    blob
}

struct CngHandles {
    rsa_algo: Option<BCRYPT_ALG_HANDLE>,
    sha256_algo: Option<BCRYPT_ALG_HANDLE>,
    key: Option<BCRYPT_KEY_HANDLE>,
    hash: Option<BCRYPT_HASH_HANDLE>,
}

impl Drop for CngHandles {
    fn drop(&mut self) {
        unsafe {
            if let Some(handle) = self.rsa_algo {
                let _ = BCryptCloseAlgorithmProvider(handle, 0);
            }
            if let Some(handle) = self.sha256_algo {
                let _ = BCryptCloseAlgorithmProvider(handle, 0);
            }
            if let Some(handle) = self.key {
                let _ = BCryptDestroyKey(handle);
            }
            if let Some(handle) = self.hash {
                let _ = BCryptDestroyHash(handle);
            }
        }
    }
}

pub fn sign_rsa_pkcs1_sha256(
    modulus: &[u8],
    public_exponent: &[u8],
    prime1: &[u8],
    prime2: &[u8],
    message: &[u8],
) -> Result<Vec<u8>> {
    let mut handles = CngHandles {
        rsa_algo: None,
        sha256_algo: None,
        key: None,
        hash: None,
    };

    // Open RSA algorithm provider
    let mut h_rsa_algo = BCRYPT_ALG_HANDLE::default();
    unsafe {
        BCryptOpenAlgorithmProvider(
            &mut h_rsa_algo,
            BCRYPT_RSA_ALGORITHM,
            None,
            BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS::default(),
        )
        .ok()?;
    }
    handles.rsa_algo = Some(h_rsa_algo);

    // Open SHA-256 algorithm provider
    let mut h_sha256_algo = BCRYPT_ALG_HANDLE::default();
    unsafe {
        BCryptOpenAlgorithmProvider(
            &mut h_sha256_algo,
            BCRYPT_SHA256_ALGORITHM,
            None,
            BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS::default(),
        )
        .ok()?;
    }
    handles.sha256_algo = Some(h_sha256_algo);

    // Build the key blob and import the key pair
    let blob = build_rsa_private_blob(modulus, public_exponent, prime1, prime2);
    let mut key_handle = BCRYPT_KEY_HANDLE::default();
    unsafe {
        BCryptImportKeyPair(
            h_rsa_algo,
            None,
            BCRYPT_RSAPRIVATE_BLOB,
            &mut key_handle,
            &blob,
            0,
        )
        .ok()?;
    }
    handles.key = Some(key_handle);

    // Create a hash object for SHA-256
    let mut hash_handle = BCRYPT_HASH_HANDLE::default();
    unsafe {
        BCryptCreateHash(h_sha256_algo, &mut hash_handle, None, None, 0).ok()?;
    }
    handles.hash = Some(hash_handle);

    // Hash the message
    unsafe {
        BCryptHashData(hash_handle, message, 0).ok()?;
    }

    // Finish the hash
    let mut hash = [0u8; 32];
    unsafe {
        BCryptFinishHash(hash_handle, &mut hash, 0).ok()?;
    }

    // Get the signature size
    let mut signature_size = 0u32;
    unsafe {
        BCryptSignHash(
            key_handle,
            None,
            &hash,
            None,
            &mut signature_size,
            BCRYPT_PAD_PKCS1,
        )
        .ok()?;
    }

    // Allocate the signature buffer and sign
    let mut signature = vec![0u8; signature_size as usize];
    unsafe {
        BCryptSignHash(
            key_handle,
            None,
            &hash,
            Some(&mut signature),
            &mut signature_size,
            BCRYPT_PAD_PKCS1,
        )
        .ok()?;
    }

    Ok(signature)
}
