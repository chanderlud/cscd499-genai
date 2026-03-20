use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, HRESULT, PWSTR};
use windows::Win32::Foundation::{SEC_E_OK, SEC_I_COMPLETE_AND_CONTINUE, SEC_I_COMPLETE_NEEDED};
use windows::Win32::Security::Authentication::Identity::{
    AcceptSecurityContext, AcquireCredentialsHandleW, CompleteAuthToken, DecryptMessage,
    DeleteSecurityContext, EncryptMessage, FreeContextBuffer, FreeCredentialsHandle,
    InitializeSecurityContextW, QueryContextAttributesW, QuerySecurityPackageInfoW, SecBuffer,
    SecBufferDesc, SecPkgContext_Sizes, ASC_REQ_CONFIDENTIALITY, ASC_REQ_CONNECTION, ASC_REQ_FLAGS,
    ASC_REQ_INTEGRITY, ASC_REQ_REPLAY_DETECT, ASC_REQ_SEQUENCE_DETECT, ISC_REQ_CONFIDENTIALITY,
    ISC_REQ_CONNECTION, ISC_REQ_FLAGS, ISC_REQ_INTEGRITY, ISC_REQ_REPLAY_DETECT,
    ISC_REQ_SEQUENCE_DETECT, SECBUFFER_DATA, SECBUFFER_TOKEN, SECBUFFER_VERSION, SECPKG_ATTR_SIZES,
    SECPKG_CRED, SECPKG_CRED_INBOUND, SECPKG_CRED_OUTBOUND, SECURITY_NATIVE_DREP,
};
use windows::Win32::Security::Credentials::SecHandle;

fn wide_null(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(once(0)).collect()
}

#[derive(Debug)]
struct SafeCredHandle(SecHandle);

impl Drop for SafeCredHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = FreeCredentialsHandle(&self.0);
        }
    }
}

#[derive(Debug)]
struct SafeCtxtHandle(SecHandle);

impl Drop for SafeCtxtHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteSecurityContext(&self.0);
        }
    }
}

#[derive(Debug, Clone)]
struct EncryptedMessage {
    token: Vec<u8>,
    data: Vec<u8>,
}

fn acquire_credentials(package: &str, credential_use: u32) -> Result<SafeCredHandle> {
    let package_name = wide_null(package);
    let mut cred_handle = SecHandle::default();
    let mut timestamp = 0i64;

    unsafe {
        AcquireCredentialsHandleW(
            None,
            PWSTR(package_name.as_ptr() as *mut _),
            SECPKG_CRED(credential_use),
            None,
            None,
            None,
            None,
            &mut cred_handle,
            Some(&mut timestamp),
        )?;
    }

    Ok(SafeCredHandle(cred_handle))
}

fn query_max_token(package: &str) -> Result<u32> {
    let package_name = wide_null(package);
    unsafe {
        let pkg_info = QuerySecurityPackageInfoW(PWSTR(package_name.as_ptr() as *mut _))?;

        if pkg_info.is_null() {
            return Err(Error::new(
                HRESULT(0x80004005u32 as i32),
                "QuerySecurityPackageInfoW returned null package info",
            ));
        }

        let max_token = (*pkg_info).cbMaxToken;
        FreeContextBuffer(pkg_info as *mut _)?;
        Ok(max_token)
    }
}

fn has_context(handle: &SecHandle) -> bool {
    *handle != SecHandle::default()
}

fn maybe_complete_auth_token(
    context: &SecHandle,
    output_desc: &mut SecBufferDesc,
    status: HRESULT,
) -> Result<()> {
    if status == SEC_I_COMPLETE_NEEDED || status == SEC_I_COMPLETE_AND_CONTINUE {
        unsafe {
            CompleteAuthToken(context as *const _, output_desc as *mut _)?;
        }
    }
    Ok(())
}

fn initialize_security_context(
    cred_handle: &SecHandle,
    context: &mut SecHandle,
    target_name: Option<&str>,
    context_req: u32,
    input_token: &[u8],
    max_token: u32,
) -> Result<(HRESULT, u32, Vec<u8>)> {
    let target_name_wide = target_name.map(wide_null);
    let existing_context = if has_context(context) {
        Some(context as *const _)
    } else {
        None
    };

    let mut context_attr = 0u32;
    let mut timestamp = 0i64;

    let mut input_buffer = SecBuffer {
        pvBuffer: input_token.as_ptr() as *mut _,
        cbBuffer: input_token.len() as u32,
        BufferType: SECBUFFER_TOKEN,
    };
    let input_desc = SecBufferDesc {
        ulVersion: SECBUFFER_VERSION,
        cBuffers: 1,
        pBuffers: &mut input_buffer,
    };

    let mut output_token = vec![0u8; max_token as usize];
    let mut output_buffer = SecBuffer {
        pvBuffer: output_token.as_mut_ptr() as *mut _,
        cbBuffer: max_token,
        BufferType: SECBUFFER_TOKEN,
    };
    let mut desc = SecBufferDesc {
        ulVersion: SECBUFFER_VERSION,
        cBuffers: 1,
        pBuffers: &mut output_buffer,
    };

    let status = unsafe {
        InitializeSecurityContextW(
            Some(cred_handle as *const _),
            existing_context,
            target_name_wide.as_ref().map(|w| w.as_ptr()),
            ISC_REQ_FLAGS(context_req),
            0,
            SECURITY_NATIVE_DREP,
            if input_token.is_empty() {
                None
            } else {
                Some(&input_desc as *const _ as *mut _)
            },
            0,
            Some(context as *mut _),
            Some(&desc as *const _ as *mut _),
            &mut context_attr,
            Some(&mut timestamp),
        )
    };

    maybe_complete_auth_token(context, &mut desc, status)?;

    if status.is_err() {
        return Err(Error::from_hresult(status));
    }

    output_token.truncate(output_buffer.cbBuffer as usize);
    Ok((status, context_attr, output_token))
}

fn accept_security_context(
    cred_handle: &SecHandle,
    context: &mut SecHandle,
    input_token: &[u8],
    context_req: u32,
    max_token: u32,
) -> Result<(HRESULT, u32, Vec<u8>)> {
    let existing_context = if has_context(context) {
        Some(context as *const _)
    } else {
        None
    };

    let mut context_attr = 0u32;
    let mut timestamp = 0i64;

    let mut input_buffer = SecBuffer {
        pvBuffer: input_token.as_ptr() as *mut _,
        cbBuffer: input_token.len() as u32,
        BufferType: SECBUFFER_TOKEN,
    };
    let input_desc = SecBufferDesc {
        ulVersion: SECBUFFER_VERSION,
        cBuffers: 1,
        pBuffers: &mut input_buffer,
    };

    let mut output_token = vec![0u8; max_token as usize];
    let mut output_buffer = SecBuffer {
        pvBuffer: output_token.as_mut_ptr() as *mut _,
        cbBuffer: max_token,
        BufferType: SECBUFFER_TOKEN,
    };
    let mut desc = SecBufferDesc {
        ulVersion: SECBUFFER_VERSION,
        cBuffers: 1,
        pBuffers: &mut output_buffer,
    };

    let status = unsafe {
        AcceptSecurityContext(
            Some(cred_handle as *const _),
            existing_context,
            if input_token.is_empty() {
                None
            } else {
                Some(&input_desc as *const _ as *mut _)
            },
            ASC_REQ_FLAGS(context_req),
            SECURITY_NATIVE_DREP,
            Some(context as *mut _),
            Some(&desc as *const _ as *mut _),
            &mut context_attr,
            Some(&mut timestamp),
        )
    };

    maybe_complete_auth_token(context, &mut desc, status)?;

    if status.is_err() {
        return Err(Error::from_hresult(status));
    }

    output_token.truncate(output_buffer.cbBuffer as usize);
    Ok((status, context_attr, output_token))
}

fn query_context_sizes(context: &SecHandle) -> Result<SecPkgContext_Sizes> {
    let mut sizes = SecPkgContext_Sizes::default();
    unsafe {
        QueryContextAttributesW(
            context as *const _,
            SECPKG_ATTR_SIZES,
            &mut sizes as *mut _ as *mut _,
        )?;
    }
    Ok(sizes)
}

fn encrypt_message(
    context: &SecHandle,
    plaintext: &[u8],
    sizes: &SecPkgContext_Sizes,
) -> Result<EncryptedMessage> {
    let mut data = plaintext.to_vec();
    let mut token = vec![0u8; sizes.cbSecurityTrailer as usize];

    let mut buffers = [
        SecBuffer {
            cbBuffer: token.len() as u32,
            BufferType: SECBUFFER_TOKEN,
            pvBuffer: token.as_mut_ptr() as *mut _,
        },
        SecBuffer {
            cbBuffer: data.len() as u32,
            BufferType: SECBUFFER_DATA,
            pvBuffer: data.as_mut_ptr() as *mut _,
        },
    ];

    let desc = SecBufferDesc {
        ulVersion: SECBUFFER_VERSION,
        cBuffers: buffers.len() as u32,
        pBuffers: buffers.as_mut_ptr(),
    };

    unsafe {
        let status = EncryptMessage(context as *const _, 0, &desc as *const _ as *mut _, 0);
        if status.is_err() {
            return Err(Error::from_hresult(status));
        }
    }

    token.truncate(buffers[0].cbBuffer as usize);
    data.truncate(buffers[1].cbBuffer as usize);

    Ok(EncryptedMessage { token, data })
}

fn decrypt_message(context: &SecHandle, encrypted: &EncryptedMessage) -> Result<Vec<u8>> {
    let mut token = encrypted.token.clone();
    let mut data = encrypted.data.clone();

    let mut buffers = [
        SecBuffer {
            cbBuffer: token.len() as u32,
            BufferType: SECBUFFER_TOKEN,
            pvBuffer: token.as_mut_ptr() as *mut _,
        },
        SecBuffer {
            cbBuffer: data.len() as u32,
            BufferType: SECBUFFER_DATA,
            pvBuffer: data.as_mut_ptr() as *mut _,
        },
    ];

    let desc = SecBufferDesc {
        ulVersion: SECBUFFER_VERSION,
        cBuffers: buffers.len() as u32,
        pBuffers: buffers.as_mut_ptr(),
    };

    let mut qop = 0u32;

    unsafe {
        let status = DecryptMessage(
            context as *const _,
            &desc as *const _ as *mut _,
            0,
            Some(&mut qop),
        );
        if status.is_err() {
            return Err(Error::from_hresult(status));
        }
    }

    let data_buf = buffers
        .iter()
        .find(|b| b.BufferType == SECBUFFER_DATA)
        .ok_or_else(|| Error::new(HRESULT(0x80090308u32 as i32), "missing SECBUFFER_DATA"))?;

    let plaintext = unsafe {
        std::slice::from_raw_parts(data_buf.pvBuffer as *const u8, data_buf.cbBuffer as usize)
    };

    Ok(plaintext.to_vec())
}

pub fn sspi_ntlm_seal_roundtrip(plaintext: &[u8]) -> Result<Vec<u8>> {
    const CLIENT_FLAGS: u32 = ISC_REQ_CONFIDENTIALITY.0
        | ISC_REQ_INTEGRITY.0
        | ISC_REQ_REPLAY_DETECT.0
        | ISC_REQ_SEQUENCE_DETECT.0
        | ISC_REQ_CONNECTION.0;
    const SERVER_FLAGS: u32 = ASC_REQ_CONFIDENTIALITY.0
        | ASC_REQ_INTEGRITY.0
        | ASC_REQ_REPLAY_DETECT.0
        | ASC_REQ_SEQUENCE_DETECT.0
        | ASC_REQ_CONNECTION.0;

    let max_token = query_max_token("NTLM")?;

    let client_creds = acquire_credentials("NTLM", SECPKG_CRED_OUTBOUND.0)?;
    let server_creds = acquire_credentials("NTLM", SECPKG_CRED_INBOUND.0)?;

    let mut client_ctx = SecHandle::default();
    let mut server_ctx = SecHandle::default();
    let mut server_token = Vec::new();

    loop {
        let (client_status, _client_attrs, client_token) = initialize_security_context(
            &client_creds.0,
            &mut client_ctx,
            None,
            CLIENT_FLAGS,
            &server_token,
            max_token,
        )?;

        let (server_status, _server_attrs, new_server_token) = accept_security_context(
            &server_creds.0,
            &mut server_ctx,
            &client_token,
            SERVER_FLAGS,
            max_token,
        )?;
        server_token = new_server_token;

        if client_status == SEC_E_OK && server_status == SEC_E_OK {
            break;
        }
    }

    let client_ctx = SafeCtxtHandle(client_ctx);
    let server_ctx = SafeCtxtHandle(server_ctx);

    let sizes = query_context_sizes(&client_ctx.0)?;
    let encrypted = encrypt_message(&client_ctx.0, plaintext, &sizes)?;
    let decrypted = decrypt_message(&server_ctx.0, &encrypted)?;

    Ok(decrypted)
}
