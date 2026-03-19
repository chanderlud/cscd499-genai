use windows::core::{Result, HSTRING};
use windows::Data::Xml::Dom::XmlDocument;
use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};

fn show_reply_toast(app_id: &str, title: &str, message: &str, placeholder: &str) -> Result<()> {
    let xml = XmlDocument::new()?;

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

    xml.LoadXml(&HSTRING::from(xml_content))?;

    let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(app_id))?;

    let toast = ToastNotification::CreateToastNotification(&xml)?;

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

fn main() -> Result<()> {
    show_reply_toast(
        "YourAppId",
        "Notification Title",
        "This is the message body",
        "Type your reply here",
    )
}
