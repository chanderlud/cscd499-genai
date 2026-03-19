use std::time::SystemTime;
use windows::core::{Result, HSTRING};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
use windows::UI::Notifications::ToastNotificationManager;

pub struct ToastHistoryItem {
    pub tag: String,
    pub group: Option<String>,
    pub display_time: SystemTime,
}

fn get_toast_history(app_id: &str) -> Result<Vec<ToastHistoryItem>> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
    }

    let app_id_hstring = HSTRING::from(app_id);

    // Get history for specific application
    let history = ToastNotificationManager::History()?;
    let notifications = history.GetHistoryWithId(&app_id_hstring)?;

    let mut result = Vec::new();
    let count = notifications.Size()?;

    for i in 0..count {
        let notification = notifications.GetAt(i)?;

        let tag = notification.Tag()?.to_string();
        let group = notification.Group()?.to_string();

        // Note: ToastNotification doesn't have a Timestamp method
        // Using SystemTime::now() as placeholder since Windows API doesn't provide display time
        let display_time = SystemTime::now();

        let group_option = if group.is_empty() { None } else { Some(group) };

        result.push(ToastHistoryItem {
            tag,
            group: group_option,
            display_time,
        });
    }

    Ok(result)
}

fn main() {
    let app_id = "Your.App.ID";
    match get_toast_history(app_id) {
        Ok(notifications) => {
            println!("Found {} notifications", notifications.len());
            for notification in notifications {
                println!("Tag: {}, Group: {:?}", notification.tag, notification.group);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
