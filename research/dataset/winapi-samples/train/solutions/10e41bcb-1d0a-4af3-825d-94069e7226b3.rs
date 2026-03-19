use windows::core::{Result, HSTRING};
use windows::Data::Xml::Dom::XmlDocument;
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};

/// Represents a toast notification with title and text content
pub struct Toast {
    title: String,
    text1: String,
}

impl Toast {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            text1: String::new(),
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn text1(mut self, text1: &str) -> Self {
        self.text1 = text1.to_string();
        self
    }

    /// Convert Toast to XmlDocument for WinRT API
    fn to_xml(&self) -> Result<XmlDocument> {
        let xml = format!(
            r#"<toast>
                <visual>
                    <binding template="ToastGeneric">
                        <text>{}</text>
                        <text>{}</text>
                    </binding>
                </visual>
            </toast>"#,
            self.title, self.text1
        );

        let document = XmlDocument::new()?;
        document.LoadXml(&HSTRING::from(xml))?;
        Ok(document)
    }
}

impl Default for Toast {
    fn default() -> Self {
        Self::new()
    }
}

/// Updates an existing toast notification with new content
pub fn update_toast(app_id: &str, tag: &str, new_toast: Toast) -> Result<()> {
    // Initialize COM for this thread
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
    }

    // Ensure we uninitialize COM when done
    struct ComGuard;
    impl Drop for ComGuard {
        fn drop(&mut self) {
            unsafe { CoUninitialize() };
        }
    }
    let _guard = ComGuard;

    // Get toast notifier for the application
    let app_id_hstring = HSTRING::from(app_id);
    let notifier = ToastNotificationManager::CreateToastNotifierWithId(&app_id_hstring)?;

    // Create new toast notification from provided Toast
    let toast_xml = new_toast.to_xml()?;
    let new_notification = ToastNotification::CreateToastNotification(&toast_xml)?;

    // Set the tag on the new notification
    let tag_hstring = HSTRING::from(tag);
    new_notification.SetTag(&tag_hstring)?;

    // Show the new notification - this will replace any existing notification with the same tag
    notifier.Show(&new_notification)?;

    Ok(())
}

/// Helper function to show a toast notification with a specific tag
pub fn show_toast_with_tag(app_id: &str, tag: &str, toast: Toast) -> Result<()> {
    // Initialize COM for this thread
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
    }

    // Ensure we uninitialize COM when done
    struct ComGuard;
    impl Drop for ComGuard {
        fn drop(&mut self) {
            unsafe { CoUninitialize() };
        }
    }
    let _guard = ComGuard;

    // Get toast notifier for the application
    let app_id_hstring = HSTRING::from(app_id);
    let notifier = ToastNotificationManager::CreateToastNotifierWithId(&app_id_hstring)?;

    // Create toast notification
    let toast_xml = toast.to_xml()?;
    let notification = ToastNotification::CreateToastNotification(&toast_xml)?;

    // Set the tag on the notification
    let tag_hstring = HSTRING::from(tag);
    notification.SetTag(&tag_hstring)?;

    // Show the notification
    notifier.Show(&notification)?;

    Ok(())
}
