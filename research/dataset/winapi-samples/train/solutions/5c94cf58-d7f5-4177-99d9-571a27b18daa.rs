use std::time::{SystemTime, UNIX_EPOCH};
use windows::core::{Result, HSTRING};
use windows::Data::Xml::Dom::XmlDocument;
use windows::Foundation::DateTime;
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
use windows::UI::Notifications::{ScheduledToastNotification, ToastNotificationManager};

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn system_time_to_datetime(time: SystemTime) -> DateTime {
    let duration = time.duration_since(UNIX_EPOCH).unwrap();
    // Windows epoch is 1601-01-01, which is 11644473600 seconds before UNIX epoch
    let windows_epoch_offset: u64 = 11644473600;
    let total_seconds = duration.as_secs() + windows_epoch_offset;
    let nanos = duration.subsec_nanos();

    // Convert to 100-nanosecond intervals
    let universal_time = (total_seconds as i64) * 10_000_000 + (nanos as i64) / 100;

    DateTime {
        UniversalTime: universal_time,
    }
}

pub fn schedule_toast(
    app_id: &str,
    title: &str,
    body: &str,
    delay: std::time::Duration,
) -> Result<()> {
    // Initialize COM with apartment threading
    // SAFETY: Calling CoInitializeEx is safe as we're using valid parameters
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
    }

    // Ensure CoUninitialize is called when function returns
    struct ComGuard;
    impl Drop for ComGuard {
        fn drop(&mut self) {
            // SAFETY: CoUninitialize is safe to call after successful CoInitializeEx
            unsafe { CoUninitialize() };
        }
    }
    let _guard = ComGuard;

    // Escape XML content
    let escaped_title = escape_xml(title);
    let escaped_body = escape_xml(body);

    // Create XML template for toast notification
    let xml_content = format!(
        r#"<toast duration="short">
            <visual>
                <binding template="ToastGeneric">
                    <text>{}</text>
                    <text>{}</text>
                </binding>
            </visual>
        </toast>"#,
        escaped_title, escaped_body
    );

    // Create and load XML document
    let xml_doc = XmlDocument::new()?;
    xml_doc.LoadXml(&HSTRING::from(xml_content))?;

    // Calculate scheduled time
    let scheduled_time = SystemTime::now() + delay;
    let datetime = system_time_to_datetime(scheduled_time);

    // Create scheduled toast notification from XML with delivery time
    let notification =
        ScheduledToastNotification::CreateScheduledToastNotification(&xml_doc, datetime)?;

    // Create toast notifier for the app and schedule notification
    let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(app_id))?;
    notifier.AddToSchedule(&notification)?;

    Ok(())
}
