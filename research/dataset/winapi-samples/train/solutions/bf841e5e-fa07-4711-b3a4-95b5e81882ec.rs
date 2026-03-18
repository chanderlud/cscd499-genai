use std::mem;
use windows::core::{Error, Result};
use windows::Win32::Foundation::ERROR_INVALID_DATA;
use windows::Win32::Security::Cryptography::{
    BCryptExportKey, BCRYPT_KEY_HANDLE, BCRYPT_RSAFULLPRIVATE_BLOB, BCRYPT_RSAKEY_BLOB,
    BCRYPT_RSAPRIVATE_MAGIC,
};

fn encode_der_length(length: usize) -> Vec<u8> {
    if length < 0x80 {
        vec![length as u8]
    } else if length < 0x100 {
        vec![0x81, length as u8]
    } else if length < 0x10000 {
        vec![0x82, (length >> 8) as u8, length as u8]
    } else {
        panic!("DER length too large");
    }
}

fn encode_der_integer(value: &[u8]) -> Vec<u8> {
    let mut content = Vec::new();

    // Remove leading zeros, but keep at least one byte
    let mut start = 0;
    while start < value.len() - 1 && value[start] == 0 {
        start += 1;
    }
    let trimmed = &value[start..];

    // If high bit is set, prepend 0x00 to indicate positive integer
    if !trimmed.is_empty() && (trimmed[0] & 0x80) != 0 {
        content.push(0x00);
    }
    content.extend_from_slice(trimmed);

    // Build TLV (Tag-Length-Value)
    let mut result = vec![0x02]; // INTEGER tag
    result.extend(encode_der_length(content.len()));
    result.extend(content);
    result
}

fn encode_der_sequence(items: &[Vec<u8>]) -> Vec<u8> {
    let content: Vec<u8> = items.iter().flatten().copied().collect();
    let mut result = vec![0x30]; // SEQUENCE tag
    result.extend(encode_der_length(content.len()));
    result.extend(content);
    result
}

pub fn export_rsa_private_key_to_der(
    key_handle: BCRYPT_KEY_HANDLE,
) -> windows::core::Result<Vec<u8>> {
    // First, get the required buffer size
    let mut result_size = 0u32;
    let status = unsafe {
        BCryptExportKey(
            key_handle,
            None,
            BCRYPT_RSAFULLPRIVATE_BLOB,
            None,
            &mut result_size,
            0,
        )
    };

    if status.0 != 0 {
        return Err(Error::from_hresult(status.into()));
    }

    // Allocate buffer and export the key
    let mut buffer = vec![0u8; result_size as usize];
    let status = unsafe {
        BCryptExportKey(
            key_handle,
            None,
            BCRYPT_RSAFULLPRIVATE_BLOB,
            Some(buffer.as_mut_slice()),
            &mut result_size,
            0,
        )
    };

    if status.0 != 0 {
        return Err(Error::from_hresult(status.into()));
    }

    // Parse the BCRYPT_RSAKEY_BLOB header
    if buffer.len() < mem::size_of::<BCRYPT_RSAKEY_BLOB>() {
        return Err(Error::from_hresult(ERROR_INVALID_DATA.to_hresult()));
    }

    let header = unsafe { &*(buffer.as_ptr() as *const BCRYPT_RSAKEY_BLOB) };

    // Verify it's a full private key blob
    if header.Magic != BCRYPT_RSAPRIVATE_MAGIC {
        return Err(Error::from_hresult(ERROR_INVALID_DATA.to_hresult()));
    }

    // Calculate offsets for each component
    let header_size = mem::size_of::<BCRYPT_RSAKEY_BLOB>();
    let mut offset = header_size;

    // Helper to extract a component and advance offset
    let extract_component = |offset: &mut usize, size: usize| -> Result<&[u8]> {
        if *offset + size > buffer.len() {
            return Err(Error::from_hresult(ERROR_INVALID_DATA.to_hresult()));
        }
        let component = &buffer[*offset..*offset + size];
        *offset += size;
        Ok(component)
    };

    // Extract components in the order they appear in the blob
    let public_exp = extract_component(&mut offset, header.cbPublicExp as usize)?;
    let modulus = extract_component(&mut offset, header.cbModulus as usize)?;
    let prime1 = extract_component(&mut offset, header.cbPrime1 as usize)?;
    let prime2 = extract_component(&mut offset, header.cbPrime2 as usize)?;
    let private_exp = extract_component(&mut offset, header.cbModulus as usize)?;
    let exponent1 = extract_component(&mut offset, header.cbPrime1 as usize)?;
    let exponent2 = extract_component(&mut offset, header.cbPrime2 as usize)?;
    let coefficient = extract_component(&mut offset, header.cbPrime1 as usize)?;

    // Encode each component as DER INTEGER
    let version = encode_der_integer(&[0]); // Version 0 for multi-prime RSA
    let modulus_der = encode_der_integer(modulus);
    let public_exp_der = encode_der_integer(public_exp);
    let private_exp_der = encode_der_integer(private_exp);
    let prime1_der = encode_der_integer(prime1);
    let prime2_der = encode_der_integer(prime2);
    let exponent1_der = encode_der_integer(exponent1);
    let exponent2_der = encode_der_integer(exponent2);
    let coefficient_der = encode_der_integer(coefficient);

    // Build the RSAPrivateKey SEQUENCE
    let rsa_private_key = encode_der_sequence(&[
        version,
        modulus_der,
        public_exp_der,
        private_exp_der,
        prime1_der,
        prime2_der,
        exponent1_der,
        exponent2_der,
        coefficient_der,
    ]);

    Ok(rsa_private_key)
}
