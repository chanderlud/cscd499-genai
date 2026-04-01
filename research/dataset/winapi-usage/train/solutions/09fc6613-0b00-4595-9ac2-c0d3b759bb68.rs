use windows::core::{Error, Result};
use windows::Win32::Foundation::{GetLastError, ERROR_SUCCESS, WIN32_ERROR};
use windows::Win32::Security::Cryptography::CERT_INFO;
use windows::Win32::Security::WinTrust::WTHelperCertIsSelfSigned;

fn call_wt_helper_cert_is_self_signed() -> WIN32_ERROR {
    // Create a concrete CERT_INFO struct with default values
    let mut cert_info = CERT_INFO {
        dwVersion: 0,
        SerialNumber: Default::default(),
        SignatureAlgorithm: Default::default(),
        Issuer: Default::default(),
        NotBefore: Default::default(),
        NotAfter: Default::default(),
        SubjectPublicKeyInfo: Default::default(),
        Subject: Default::default(),
        IssuerUniqueId: Default::default(),
        SubjectUniqueId: Default::default(),
        cExtension: 0,
        rgExtension: Default::default(),
    };

    // Call the Win32 API with concrete parameter values
    // dwencoding = 0 (default encoding type)
    // Pass mutable pointer as required by the API
    let result = unsafe { WTHelperCertIsSelfSigned(0, &mut cert_info) };

    // Convert BOOL result to WIN32_ERROR
    if result.0 != 0 {
        // Success - return ERROR_SUCCESS
        ERROR_SUCCESS
    } else {
        // Failure - get error from GetLastError() and convert to WIN32_ERROR
        WIN32_ERROR(unsafe { GetLastError().0 })
    }
}
