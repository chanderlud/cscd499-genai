use std::mem::MaybeUninit;
use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::*;
use windows::Win32::Security::Authentication::Identity::*;
use windows::Win32::Security::Credentials::*;

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn sspi_ntlm_seal_roundtrip_stack(plaintext: &[u8]) -> Result<Vec<u8>> {
    // Query NTLM package info to get maximum token size
    let package_name = wide_null(std::ffi::OsStr::new("NTLM"));

    // SAFETY: FFI call with valid pointer
    let package_info_ptr = unsafe { QuerySecurityPackageInfoW(PCWSTR(package_name.as_ptr()))? };

    // SAFETY: We trust the API returned a valid pointer
    let max_token_size = unsafe { (*package_info_ptr).cbMaxToken as usize };

    // SAFETY: Free the package info buffer
    unsafe {
        FreeContextBuffer(package_info_ptr as *mut _)?;
    }

    // Get security context sizes
    let mut sizes = SecPkgContext_Sizes::default();

    // We'll get sizes after establishing contexts
    // For now, use reasonable fixed sizes based on typical NTLM
    const MAX_HANDSHAKE_STEPS: usize = 3;
    const FIXED_BUFFER_SIZE: usize = 4096; // Should be larger than max_token_size

    // Validate plaintext fits within our fixed buffer constraints
    // We need space for plaintext + security trailer + padding
    if plaintext.len() > FIXED_BUFFER_SIZE / 2 {
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_INSUFFICIENT_BUFFER.0,
        )));
    }

    // Fixed-size arrays for all intermediate buffers
    let mut client_tokens: [[u8; FIXED_BUFFER_SIZE]; MAX_HANDSHAKE_STEPS] =
        [[0u8; FIXED_BUFFER_SIZE]; MAX_HANDSHAKE_STEPS];
    let mut server_tokens: [[u8; FIXED_BUFFER_SIZE]; MAX_HANDSHAKE_STEPS] =
        [[0u8; FIXED_BUFFER_SIZE]; MAX_HANDSHAKE_STEPS];
    let mut token_sizes: [u32; MAX_HANDSHAKE_STEPS] = [0; MAX_HANDSHAKE_STEPS];

    // Acquire client credentials
    let mut client_creds = MaybeUninit::<SecHandle>::uninit();
    let mut client_creds_ts = 0i64;

    // SAFETY: FFI call with valid pointers
    unsafe {
        AcquireCredentialsHandleW(
            None,
            PCWSTR(package_name.as_ptr()),
            SECPKG_CRED_OUTBOUND,
            None,
            None,
            None,
            None,
            client_creds.as_mut_ptr(),
            Some(&mut client_creds_ts as *mut _),
        )?;
    }
    let client_creds = unsafe { client_creds.assume_init() };

    // Acquire server credentials
    let mut server_creds = MaybeUninit::<SecHandle>::uninit();
    let mut server_creds_ts = 0i64;

    // SAFETY: FFI call with valid pointers
    unsafe {
        AcquireCredentialsHandleW(
            None,
            PCWSTR(package_name.as_ptr()),
            SECPKG_CRED_INBOUND,
            None,
            None,
            None,
            None,
            server_creds.as_mut_ptr(),
            Some(&mut server_creds_ts as *mut _),
        )?;
    }
    let server_creds = unsafe { server_creds.assume_init() };

    // Initialize client context
    let mut client_ctx = MaybeUninit::<SecHandle>::uninit();
    let mut client_ctx_attrs = 0u32;
    let mut client_ctx_ts = 0i64;

    // First client token (empty)
    let mut client_buf_desc = SecBufferDesc {
        ulVersion: SECBUFFER_VERSION,
        cBuffers: 1,
        pBuffers: &mut SecBuffer {
            cbBuffer: FIXED_BUFFER_SIZE as u32,
            BufferType: SECBUFFER_TOKEN,
            pvBuffer: client_tokens[0].as_mut_ptr() as *mut _,
        },
    };

    // SAFETY: FFI call with valid pointers
    unsafe {
        InitializeSecurityContextW(
            Some(&client_creds as *const _),
            None,
            None,
            ISC_REQ_CONFIDENTIALITY | ISC_REQ_SEQUENCE_DETECT,
            0,
            SECURITY_NATIVE_DREP,
            None,
            0,
            Some(client_ctx.as_mut_ptr()),
            Some(&mut client_buf_desc as *mut _),
            &mut client_ctx_attrs,
            Some(&mut client_ctx_ts as *mut _),
        )
        .ok()?;
    }
    let mut client_ctx = unsafe { client_ctx.assume_init() };

    token_sizes[0] = unsafe { (*client_buf_desc.pBuffers).cbBuffer };

    // Server accepts client token
    let mut server_ctx = MaybeUninit::<SecHandle>::uninit();
    let mut server_ctx_attrs = 0u32;
    let mut server_ctx_ts = 0i64;

    let mut server_buf_desc = SecBufferDesc {
        ulVersion: SECBUFFER_VERSION,
        cBuffers: 1,
        pBuffers: &mut SecBuffer {
            cbBuffer: token_sizes[0],
            BufferType: SECBUFFER_TOKEN,
            pvBuffer: client_tokens[0].as_mut_ptr() as *mut _,
        },
    };

    let mut server_out_buf_desc = SecBufferDesc {
        ulVersion: SECBUFFER_VERSION,
        cBuffers: 1,
        pBuffers: &mut SecBuffer {
            cbBuffer: FIXED_BUFFER_SIZE as u32,
            BufferType: SECBUFFER_TOKEN,
            pvBuffer: server_tokens[0].as_mut_ptr() as *mut _,
        },
    };

    // SAFETY: FFI call with valid pointers
    unsafe {
        AcceptSecurityContext(
            Some(&server_creds as *const _),
            None,
            Some(&mut server_buf_desc as *mut _),
            ASC_REQ_CONFIDENTIALITY | ASC_REQ_SEQUENCE_DETECT,
            SECURITY_NATIVE_DREP,
            Some(server_ctx.as_mut_ptr()),
            Some(&mut server_out_buf_desc as *mut _),
            &mut server_ctx_attrs,
            Some(&mut server_ctx_ts as *mut _),
        )
        .ok()?;
    }
    let mut server_ctx = unsafe { server_ctx.assume_init() };

    token_sizes[1] = unsafe { (*server_out_buf_desc.pBuffers).cbBuffer };

    // Client processes server token
    let mut client_buf_desc2 = SecBufferDesc {
        ulVersion: SECBUFFER_VERSION,
        cBuffers: 1,
        pBuffers: &mut SecBuffer {
            cbBuffer: token_sizes[1],
            BufferType: SECBUFFER_TOKEN,
            pvBuffer: server_tokens[0].as_mut_ptr() as *mut _,
        },
    };

    let mut client_out_buf_desc = SecBufferDesc {
        ulVersion: SECBUFFER_VERSION,
        cBuffers: 1,
        pBuffers: &mut SecBuffer {
            cbBuffer: FIXED_BUFFER_SIZE as u32,
            BufferType: SECBUFFER_TOKEN,
            pvBuffer: client_tokens[1].as_mut_ptr() as *mut _,
        },
    };

    // SAFETY: FFI call with valid pointers
    unsafe {
        InitializeSecurityContextW(
            Some(&client_creds as *const _),
            Some(&mut client_ctx as *mut _),
            None,
            ISC_REQ_CONFIDENTIALITY | ISC_REQ_SEQUENCE_DETECT,
            0,
            SECURITY_NATIVE_DREP,
            Some(&mut client_buf_desc2 as *mut _),
            0,
            Some(&mut client_ctx as *mut _),
            Some(&mut client_out_buf_desc as *mut _),
            &mut client_ctx_attrs,
            Some(&mut client_ctx_ts as *mut _),
        )
        .ok()?;
    }

    token_sizes[2] = unsafe { (*client_out_buf_desc.pBuffers).cbBuffer };

    // Server processes final client token if needed
    if token_sizes[2] > 0 {
        let mut server_buf_desc2 = SecBufferDesc {
            ulVersion: SECBUFFER_VERSION,
            cBuffers: 1,
            pBuffers: &mut SecBuffer {
                cbBuffer: token_sizes[2],
                BufferType: SECBUFFER_TOKEN,
                pvBuffer: client_tokens[1].as_mut_ptr() as *mut _,
            },
        };

        let mut server_out_buf_desc2 = SecBufferDesc {
            ulVersion: SECBUFFER_VERSION,
            cBuffers: 1,
            pBuffers: &mut SecBuffer {
                cbBuffer: FIXED_BUFFER_SIZE as u32,
                BufferType: SECBUFFER_TOKEN,
                pvBuffer: server_tokens[1].as_mut_ptr() as *mut _,
            },
        };

        // SAFETY: FFI call with valid pointers
        unsafe {
            AcceptSecurityContext(
                Some(&server_creds as *const _),
                Some(&mut server_ctx as *mut _),
                Some(&mut server_buf_desc2 as *mut _),
                ASC_REQ_CONFIDENTIALITY | ASC_REQ_SEQUENCE_DETECT,
                SECURITY_NATIVE_DREP,
                Some(&mut server_ctx as *mut _),
                Some(&mut server_out_buf_desc2 as *mut _),
                &mut server_ctx_attrs,
                Some(&mut server_ctx_ts as *mut _),
            )
            .ok()?;
        }
    }

    // Query context sizes for encryption
    let mut sizes = SecPkgContext_Sizes::default();

    // SAFETY: FFI call with valid pointers
    unsafe {
        QueryContextAttributesW(
            &client_ctx as *const _,
            SECPKG_ATTR_SIZES,
            &mut sizes as *mut _ as *mut _,
        )?;
    }

    // Prepare buffers for encryption
    let mut encrypted_data = [0u8; FIXED_BUFFER_SIZE];
    let mut security_trailer = [0u8; FIXED_BUFFER_SIZE];
    let mut padding_buffer = [0u8; FIXED_BUFFER_SIZE];

    // Copy plaintext to encrypted_data buffer
    let plaintext_len = plaintext.len();
    if plaintext_len > encrypted_data.len() - sizes.cbSecurityTrailer as usize {
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_INSUFFICIENT_BUFFER.0,
        )));
    }
    encrypted_data[..plaintext_len].copy_from_slice(plaintext);

    // Set up buffers for EncryptMessage
    let mut encrypt_buffers = [
        SecBuffer {
            cbBuffer: plaintext_len as u32,
            BufferType: SECBUFFER_DATA,
            pvBuffer: encrypted_data.as_mut_ptr() as *mut _,
        },
        SecBuffer {
            cbBuffer: sizes.cbSecurityTrailer,
            BufferType: SECBUFFER_TOKEN,
            pvBuffer: security_trailer.as_mut_ptr() as *mut _,
        },
        SecBuffer {
            cbBuffer: 0,
            BufferType: SECBUFFER_EMPTY,
            pvBuffer: std::ptr::null_mut(),
        },
        SecBuffer {
            cbBuffer: 0,
            BufferType: SECBUFFER_EMPTY,
            pvBuffer: std::ptr::null_mut(),
        },
    ];

    let mut encrypt_buf_desc = SecBufferDesc {
        ulVersion: SECBUFFER_VERSION,
        cBuffers: 4,
        pBuffers: encrypt_buffers.as_mut_ptr(),
    };

    let mut message_seq = 0u32;

    // SAFETY: FFI call with valid pointers
    unsafe {
        EncryptMessage(
            &mut client_ctx as *mut _,
            0,
            &mut encrypt_buf_desc as *mut _,
            message_seq,
        )
        .ok()?;
    }

    // Prepare for decryption - combine encrypted data and security trailer
    let mut decrypt_data = [0u8; FIXED_BUFFER_SIZE];
    let total_encrypted_size = plaintext_len + sizes.cbSecurityTrailer as usize;
    if total_encrypted_size > decrypt_data.len() {
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_INSUFFICIENT_BUFFER.0,
        )));
    }

    // Copy encrypted data and security trailer to decrypt buffer
    decrypt_data[..plaintext_len].copy_from_slice(&encrypted_data[..plaintext_len]);
    decrypt_data[plaintext_len..total_encrypted_size]
        .copy_from_slice(&security_trailer[..sizes.cbSecurityTrailer as usize]);

    // Set up buffers for DecryptMessage
    let mut decrypt_buffers = [
        SecBuffer {
            cbBuffer: total_encrypted_size as u32,
            BufferType: SECBUFFER_DATA,
            pvBuffer: decrypt_data.as_mut_ptr() as *mut _,
        },
        SecBuffer {
            cbBuffer: 0,
            BufferType: SECBUFFER_EMPTY,
            pvBuffer: std::ptr::null_mut(),
        },
        SecBuffer {
            cbBuffer: 0,
            BufferType: SECBUFFER_EMPTY,
            pvBuffer: std::ptr::null_mut(),
        },
        SecBuffer {
            cbBuffer: 0,
            BufferType: SECBUFFER_EMPTY,
            pvBuffer: std::ptr::null_mut(),
        },
    ];

    let mut decrypt_buf_desc = SecBufferDesc {
        ulVersion: SECBUFFER_VERSION,
        cBuffers: 4,
        pBuffers: decrypt_buffers.as_mut_ptr(),
    };

    let mut qop = 0u32;

    // SAFETY: FFI call with valid pointers
    unsafe {
        DecryptMessage(
            &mut server_ctx as *const _,
            &mut decrypt_buf_desc as *const _,
            message_seq,
            Some(&mut qop as *mut _),
        )
        .ok()?;
    }

    // Find the decrypted data buffer
    let mut decrypted_data = Vec::new();
    for buffer in &decrypt_buffers {
        if buffer.BufferType == SECBUFFER_DATA {
            // SAFETY: We trust the API returned valid data
            let data = unsafe {
                std::slice::from_raw_parts(buffer.pvBuffer as *const u8, buffer.cbBuffer as usize)
            };
            decrypted_data.extend_from_slice(data);
        }
    }

    // Clean up contexts
    // SAFETY: FFI calls with valid handles
    unsafe {
        DeleteSecurityContext(&mut client_ctx as *mut _)?;
        DeleteSecurityContext(&mut server_ctx as *mut _)?;
        FreeCredentialsHandle(&client_creds as *const _)?;
        FreeCredentialsHandle(&server_creds as *const _)?;
    }

    Ok(decrypted_data)
}
