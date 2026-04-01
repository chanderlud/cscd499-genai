use windows::core::{Error, Result};
use windows::Win32::Security::Cryptography::CERT_INFO;
use windows::Win32::Security::WinTrust::WTHelperCertIsSelfSigned;

fn call_wt_helper_cert_is_self_signed() -> Result<windows::core::BOOL> {
    // Create a minimal CERT_INFO with concrete values
    // Use Default::default() to avoid non-existent fields
    let cert_info = CERT_INFO::default();

    // Call the unsafe API with concrete parameter values
    // dwencoding = 1 (X509_ASN_ENCODING)
    let result =
        unsafe { WTHelperCertIsSelfSigned(1, &cert_info as *const CERT_INFO as *mut CERT_INFO) };

    Ok(result)
}
