use windows::core::{Error, Result, HSTRING};
use windows::Data::Xml::Dom::XmlDocument;
use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};

fn show_reply_toast(app_id: &str, title: &str, message: &str, placeholder: &str) -> Result<()> {
    // Create XML document for toast notification
    let xml = XmlDocument::new()?;

    // Build XML template with proper escaping
    let xml_content = format!(
        r#"<toast activationType="foreground">
            <visual>
                <binding template="ToastGeneric">
                    <text>{}</text>
                    <text>{}</text>
                </binding>
            </visual>
            <actions>
                <input id="replyTextBox" type="text" placeHolderContent="{}" />
                <action content="Reply" activationType="foreground" arguments="reply" />
            </actions>
        </toast>"#,
        xml_escape(title),
        xml_escape(message),
        xml_escape(placeholder)
    );

    // Load XML content
    xml.LoadXml(&HSTRING::from(xml_content))?;

    // Create toast notifier for the application
    let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(app_id))?;

    // Create toast notification from XML
    let toast = ToastNotification::CreateToastNotification(&xml)?;

    // Show the notification
    notifier.Show(&toast)?;

    Ok(())
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
