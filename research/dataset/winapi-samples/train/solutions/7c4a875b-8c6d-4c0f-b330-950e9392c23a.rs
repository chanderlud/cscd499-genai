use windows::core::{Result, HSTRING};
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
use windows::UI::Notifications::ToastNotificationManager;

struct ComGuard;

impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

fn remove_toast_by_tag(app_id: &str, tag: &str, group: Option<&str>) -> Result<()> {
    // Initialize COM with apartment threading
    unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }.ok()?;
    let _guard = ComGuard;

    // Get notification history for the specified app
    let history = ToastNotificationManager::History()?;

    // Convert strings to HSTRING for WinRT
    let tag_hstring = HSTRING::from(tag);

    // Remove toast based on group presence
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
