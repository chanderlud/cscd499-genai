use windows::core::{Error, Result};
use windows::Security::Credentials::UI::{
    UserConsentVerificationResult, UserConsentVerifier, UserConsentVerifierAvailability,
};
use windows::Win32::Foundation::E_ACCESSDENIED;

pub fn request_biometric_verification(message: &str) -> Result<UserConsentVerificationResult> {
    // Check if biometric verification is available
    let availability = UserConsentVerifier::CheckAvailabilityAsync()?.join()?;

    if availability != UserConsentVerifierAvailability::Available {
        return Err(Error::new(
            E_ACCESSDENIED,
            "Biometric verifier is not available",
        ));
    }

    // Request verification with the provided message
    let message_hstring = windows::core::HSTRING::from(message);
    let result = UserConsentVerifier::RequestVerificationAsync(&message_hstring)?.join()?;

    Ok(result)
}
