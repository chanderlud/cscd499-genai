use windows::core::{Error, Result};
use windows::Win32::Foundation::STATUS_UNSUCCESSFUL;
use windows::Win32::Security::Cryptography::{
    BCryptExportKey, BCRYPT_KEY_HANDLE, BCRYPT_RSAKEY_BLOB, BCRYPT_RSAPUBLIC_BLOB,
    BCRYPT_RSAPUBLIC_MAGIC,
};

/// Encodes an ASN.1 DER length field
fn der_length(length: usize) -> Vec<u8> {
    if length < 0x80 {
        vec![length as u8]
    } else if length < 0x100 {
        vec![0x81, length as u8]
    } else if length < 0x10000 {
        vec![0x82, (length >> 8) as u8, length as u8]
    } else {
        vec![
            0x83,
            (length >> 16) as u8,
            (length >> 8) as u8,
            length as u8,
        ]
    }
}

/// Encodes an ASN.1 DER integer from big-endian bytes
fn der_integer(bytes: &[u8]) -> Vec<u8> {
    // Remove leading zeros, but keep at least one byte
    let mut start = 0;
    while start < bytes.len() - 1 && bytes[start] == 0 {
        start += 1;
    }
    let trimmed = &bytes[start..];

    // Add padding zero if high bit is set (to keep it positive)
    let needs_padding = !trimmed.is_empty() && (trimmed[0] & 0x80) != 0;
    let content_len = trimmed.len() + if needs_padding { 1 } else { 0 };

    let mut result = vec![0x02]; // INTEGER tag
    result.extend(der_length(content_len));
    if needs_padding {
        result.push(0x00);
    }
    result.extend_from_slice(trimmed);
    result
}

/// Encodes an ASN.1 DER sequence
fn der_sequence(items: &[Vec<u8>]) -> Vec<u8> {
    let content: Vec<u8> = items.iter().flat_map(|item| item.clone()).collect();
    let mut result = vec![0x30]; // SEQUENCE tag
    result.extend(der_length(content.len()));
    result.extend(content);
    result
}

/// Encodes an ASN.1 DER bit string
fn der_bit_string(bytes: &[u8]) -> Vec<u8> {
    let mut content = vec![0x00]; // No unused bits
    content.extend_from_slice(bytes);

    let mut result = vec![0x03]; // BIT STRING tag
    result.extend(der_length(content.len()));
    result.extend(content);
    result
}

/// Encodes an ASN.1 DER object identifier
fn der_oid(oid: &[u8]) -> Vec<u8> {
    let mut result = vec![0x06]; // OID tag
    result.extend(der_length(oid.len()));
    result.extend_from_slice(oid);
    result
}

/// Encodes an ASN.1 DER null value
fn der_null() -> Vec<u8> {
    vec![0x05, 0x00] // NULL tag with zero length
}

pub fn export_rsa_public_key_to_der(key_handle: BCRYPT_KEY_HANDLE) -> Result<Vec<u8>> {
    // First call to get required buffer size
    let mut result_len = 0u32;
    let status = unsafe {
        BCryptExportKey(
            key_handle,
            None,
            BCRYPT_RSAPUBLIC_BLOB,
            None,
            &mut result_len,
            0,
        )
    };

    status.ok().map_err(|e| Error::from_hresult(e.code()))?;

    // Allocate buffer and export key
    let mut buffer = vec![0u8; result_len as usize];
    let status = unsafe {
        BCryptExportKey(
            key_handle,
            None,
            BCRYPT_RSAPUBLIC_BLOB,
            Some(buffer.as_mut_slice()),
            &mut result_len,
            0,
        )
    };

    status.ok().map_err(|e| Error::from_hresult(e.code()))?;

    // Parse the exported blob
    if buffer.len() < std::mem::size_of::<BCRYPT_RSAKEY_BLOB>() {
        return Err(Error::from_hresult(STATUS_UNSUCCESSFUL.to_hresult()));
    }

    let header = unsafe { &*(buffer.as_ptr() as *const BCRYPT_RSAKEY_BLOB) };

    if header.Magic != BCRYPT_RSAPUBLIC_MAGIC {
        return Err(Error::from_hresult(STATUS_UNSUCCESSFUL.to_hresult()));
    }

    let header_size = std::mem::size_of::<BCRYPT_RSAKEY_BLOB>();
    let exp_offset = header_size;
    let exp_size = header.cbPublicExp as usize;
    let mod_offset = exp_offset + exp_size;
    let mod_size = header.cbModulus as usize;

    if buffer.len() < mod_offset + mod_size {
        return Err(Error::from_hresult(STATUS_UNSUCCESSFUL.to_hresult()));
    }

    let exponent = &buffer[exp_offset..exp_offset + exp_size];
    let modulus = &buffer[mod_offset..mod_offset + mod_size];

    // Build RSA public key structure (RFC 3279)
    let rsa_public_key = der_sequence(&[der_integer(modulus), der_integer(exponent)]);

    // Build AlgorithmIdentifier for RSA (RFC 5280)
    // OID: 1.2.840.113549.1.1.1 (rsaEncryption)
    let rsa_oid = der_oid(&[
        0x06, 0x09, 0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x01, 0x01,
    ]);
    let algorithm_identifier = der_sequence(&[rsa_oid, der_null()]);

    // Build SubjectPublicKeyInfo (RFC 5280)
    let subject_public_key = der_bit_string(&rsa_public_key);
    let subject_public_key_info = der_sequence(&[algorithm_identifier, subject_public_key]);

    Ok(subject_public_key_info)
}
