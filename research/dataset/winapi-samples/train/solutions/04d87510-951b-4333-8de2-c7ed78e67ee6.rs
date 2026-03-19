use windows::core::{Result, HSTRING};
use windows::Data::Xml::Dom::XmlDocument;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn show_protocol_toast(
    app_id: &str,
    title: &str,
    message: &str,
    button_text: &str,
    protocol_uri: &str,
) -> Result<()> {
    // Initialize COM for the current thread
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
    }

    // Escape XML content
    let escaped_title = escape_xml(title);
    let escaped_message = escape_xml(message);
    let escaped_button_text = escape_xml(button_text);
    let escaped_protocol_uri = escape_xml(protocol_uri);

    // Build toast XML with protocol activation button
    let xml_content = format!(
        r#"<toast activationType="protocol">
            <visual>
                <binding template="ToastGeneric">
                    <text>{}</text>
                    <text>{}</text>
                </binding>
            </visual>
            <actions>
                <action
                    activationType="protocol"
                    arguments="{}"
                    content="{}"
                />
            </actions>
        </toast>"#,
        escaped_title, escaped_message, escaped_protocol_uri, escaped_button_text
    );

    // Create XML document
    let xml_doc = XmlDocument::new()?;
    xml_doc.LoadXml(&HSTRING::from(xml_content))?;

    // Create toast notification
    let toast = ToastNotification::CreateToastNotification(&xml_doc)?;

    // Create toast notifier for the specified app
    let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(app_id))?;

    // Show the notification
    notifier.Show(&toast)?;

    Ok(())
}
