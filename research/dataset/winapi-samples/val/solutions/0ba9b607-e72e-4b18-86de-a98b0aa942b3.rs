use windows::core::{Error, Interface, Result};
use windows::Foundation::{DateTime, IReference, PropertyValue};
use windows::Win32::Foundation::E_OUTOFMEMORY;
use windows::Win32::System::SystemInformation::GetSystemTimeAsFileTime;
use windows::UI::Notifications::ToastNotification;

fn set_notification_expiration(
    notification: &ToastNotification,
    minutes_from_now: u32,
) -> Result<()> {
    // Get current time as FILETIME (100-nanosecond intervals since 1601-01-01)
    // SAFETY: Function only reads system time and returns it
    let file_time = unsafe { GetSystemTimeAsFileTime() };

    // Convert FILETIME to u64 for arithmetic
    let current_time: u64 =
        ((file_time.dwHighDateTime as u64) << 32) | file_time.dwLowDateTime as u64;

    // Convert minutes to 100-nanosecond intervals (1 minute = 60 * 10,000,000)
    let minutes_in_100ns: u64 = (minutes_from_now as u64) * 60 * 10_000_000;

    // Calculate expiration time
    let expiration_time = current_time
        .checked_add(minutes_in_100ns)
        .ok_or_else(|| Error::from_hresult(E_OUTOFMEMORY))?;

    // Create DateTime from the calculated time
    let expiration_datetime = DateTime {
        UniversalTime: expiration_time as i64,
    };

    // Convert DateTime to IReference<DateTime> for the notification API
    let inspectable = PropertyValue::CreateDateTime(expiration_datetime)?;
    let expiration_reference: IReference<DateTime> = inspectable.cast()?;

    // Set expiration on the notification
    notification.SetExpirationTime(Some(&expiration_reference))
}
