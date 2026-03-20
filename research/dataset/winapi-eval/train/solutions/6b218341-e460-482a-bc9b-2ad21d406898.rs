use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, PCWSTR, PSTR};
use windows::Win32::Security::Cryptography::*;

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn generate_random_container_name() -> Vec<u16> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos(); // slightly less collision-prone than millis
    let name = format!("HunterAlpha_{}", timestamp);
    wide_null(OsStr::new(&name))
}

fn encode_subject_name(subject: &str) -> Result<Vec<u8>> {
    let subject_w = wide_null(OsStr::new(subject));
    let mut encoded_len = 0u32;

    unsafe {
        CertStrToNameW(
            X509_ASN_ENCODING,
            PCWSTR(subject_w.as_ptr()),
            CERT_X500_NAME_STR,
            None,
            None,
            &mut encoded_len,
            None,
        )?;
    }

    let mut encoded = vec![0u8; encoded_len as usize];

    unsafe {
        CertStrToNameW(
            X509_ASN_ENCODING,
            PCWSTR(subject_w.as_ptr()),
            CERT_X500_NAME_STR,
            None,
            Some(encoded.as_mut_ptr()),
            &mut encoded_len,
            None,
        )?;
    }

    encoded.truncate(encoded_len as usize);
    Ok(encoded)
}

struct CryptContext {
    prov: usize,
    key: usize,
    cert: *const CERT_CONTEXT,
    container_name: Vec<u16>,
}

impl CryptContext {
    fn new() -> Result<Self> {
        let container_name = generate_random_container_name();
        let mut prov = 0usize;

        unsafe {
            CryptAcquireContextW(
                &mut prov,
                PCWSTR(container_name.as_ptr()),
                None,
                PROV_RSA_FULL,
                CRYPT_NEWKEYSET,
            )?;
        }

        let mut key = 0usize;
        unsafe {
            let flags = CRYPT_EXPORTABLE.0 | (2048u32 << 16);

            if let Err(e) = CryptGenKey(prov, CALG_RSA_SIGN, CRYPT_KEY_FLAGS(flags), &mut key) {
                let _ = CryptReleaseContext(prov, 0);

                let mut tmp = 0usize;
                let _ = CryptAcquireContextW(
                    &mut tmp,
                    PCWSTR(container_name.as_ptr()),
                    None,
                    PROV_RSA_FULL,
                    CRYPT_DELETEKEYSET,
                );

                return Err(e);
            }
        }

        let subject_der = encode_subject_name("CN=HunterAlpha")?;
        let subject = CRYPT_INTEGER_BLOB {
            cbData: subject_der.len() as u32,
            pbData: subject_der.as_ptr() as *mut u8,
        };

        let cert = unsafe {
            CertCreateSelfSignCertificate(
                Some(HCRYPTPROV_OR_NCRYPT_KEY_HANDLE(prov)),
                &subject,
                CERT_CREATE_SELFSIGN_FLAGS(0),
                None,
                None,
                None,
                None,
                None,
            )
        };

        if cert.is_null() {
            let err = Error::from_thread();
            unsafe {
                let _ = CryptDestroyKey(key);
                let _ = CryptReleaseContext(prov, 0);

                let mut tmp = 0usize;
                let _ = CryptAcquireContextW(
                    &mut tmp,
                    PCWSTR(container_name.as_ptr()),
                    None,
                    PROV_RSA_FULL,
                    CRYPT_DELETEKEYSET,
                );
            }
            return Err(err);
        }

        Ok(Self {
            prov,
            key,
            cert,
            container_name,
        })
    }
}

impl Drop for CryptContext {
    fn drop(&mut self) {
        unsafe {
            if !self.cert.is_null() {
                let _ = CertFreeCertificateContext(Some(self.cert));
            }
            if self.key != 0 {
                let _ = CryptDestroyKey(self.key);
            }
            if self.prov != 0 {
                let _ = CryptReleaseContext(self.prov, 0);
            }

            let mut tmp = 0usize;
            let _ = CryptAcquireContextW(
                &mut tmp,
                PCWSTR(self.container_name.as_ptr()),
                None,
                PROV_RSA_FULL,
                CRYPT_DELETEKEYSET,
            );
        }
    }
}

pub fn crypt32_sign_verify_roundtrip(message: &[u8]) -> Result<Vec<u8>> {
    let ctx = CryptContext::new()?;

    let mut signed_blob_size = 0u32;
    let message_ptrs = [message.as_ptr()];
    let message_sizes = [message.len() as u32];

    let mut msg_certs: [*mut CERT_CONTEXT; 1] = [ctx.cert as *mut CERT_CONTEXT];

    let sign_para = CRYPT_SIGN_MESSAGE_PARA {
        cbSize: std::mem::size_of::<CRYPT_SIGN_MESSAGE_PARA>() as u32,
        dwMsgEncodingType: (X509_ASN_ENCODING | PKCS_7_ASN_ENCODING).0,
        pSigningCert: ctx.cert,
        HashAlgorithm: CRYPT_ALGORITHM_IDENTIFIER {
            pszObjId: PSTR(c"1.2.840.113549.1.1.5".as_ptr() as *mut u8),
            Parameters: Default::default(),
        },
        cMsgCert: msg_certs.len() as u32,
        rgpMsgCert: msg_certs.as_mut_ptr(),
        ..Default::default()
    };

    unsafe {
        CryptSignMessage(
            &sign_para,
            false,
            1,
            Some(message_ptrs.as_ptr()),
            message_sizes.as_ptr(),
            None,
            &mut signed_blob_size,
        )?;
    }

    let mut signed_blob = vec![0u8; signed_blob_size as usize];

    unsafe {
        CryptSignMessage(
            &sign_para,
            false,
            1,
            Some(message_ptrs.as_ptr()),
            message_sizes.as_ptr(),
            Some(signed_blob.as_mut_ptr()),
            &mut signed_blob_size,
        )?;
    }

    signed_blob.truncate(signed_blob_size as usize);

    let verify_para = CRYPT_VERIFY_MESSAGE_PARA {
        cbSize: std::mem::size_of::<CRYPT_VERIFY_MESSAGE_PARA>() as u32,
        dwMsgAndCertEncodingType: (X509_ASN_ENCODING | PKCS_7_ASN_ENCODING).0,
        ..Default::default()
    };

    let mut recovered_size = 0u32;

    unsafe {
        CryptVerifyMessageSignature(
            &verify_para,
            0,
            &signed_blob,
            None,
            Some(&mut recovered_size),
            None,
        )?;
    }

    let mut recovered_content = vec![0u8; recovered_size as usize];

    unsafe {
        CryptVerifyMessageSignature(
            &verify_para,
            0,
            &signed_blob,
            Some(recovered_content.as_mut_ptr()),
            Some(&mut recovered_size),
            None,
        )?;
    }

    recovered_content.truncate(recovered_size as usize);
    Ok(recovered_content)
}
