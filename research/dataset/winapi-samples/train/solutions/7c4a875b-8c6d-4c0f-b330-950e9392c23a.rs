use windows::core::{Result, HSTRING};
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
use windows::UI::Notifications::ToastNotificationManager;

struct ComGuard;

impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

fn remove_toast_by_tag(_app_id: &str, tag: &str, group: Option<&str>) -> Result<()> {
    unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }.ok()?;
    let _guard = ComGuard;

    let history = ToastNotificationManager::History()?;

    let tag_hstring = HSTRING::from(tag);

    match group {
        Some(group_str) => {
            let group_hstring = HSTRING::from(group_str);
            history.RemoveGroupedTag(&tag_hstring, &group_hstring)?;
        }
        None => {
            history.Remove(&tag_hstring)?;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    remove_toast_by_tag("MyApp", "notification_tag", Some("notification_group"))
}
