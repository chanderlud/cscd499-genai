use windows::core::Result;
use windows::Security::Credentials::UI::{UserConsentVerifier, UserConsentVerifierAvailability};

#[derive(Debug, Clone, PartialEq)]
pub enum BiometricStatus {
    Available,
    DeviceNotPresent,
    NotConfigured,
    DisabledByPolicy,
    DeviceBusy,
    Unknown,
}

pub fn check_biometric_availability() -> Result<BiometricStatus> {
    // Call the async API to check availability
    let async_operation = UserConsentVerifier::CheckAvailabilityAsync()?;

    // Wait for the async operation to complete
    let availability = async_operation.join()?;

    // Map the Windows API result to our custom enum
    Ok(match availability {
        UserConsentVerifierAvailability::Available => BiometricStatus::Available,
        UserConsentVerifierAvailability::DeviceNotPresent => BiometricStatus::DeviceNotPresent,
        UserConsentVerifierAvailability::NotConfiguredForUser => BiometricStatus::NotConfigured,
        UserConsentVerifierAvailability::DisabledByPolicy => BiometricStatus::DisabledByPolicy,
        UserConsentVerifierAvailability::DeviceBusy => BiometricStatus::DeviceBusy,
        _ => BiometricStatus::Unknown,
    })
}
