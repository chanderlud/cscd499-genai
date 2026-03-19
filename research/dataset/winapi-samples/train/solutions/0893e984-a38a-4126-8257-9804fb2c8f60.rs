use windows::core::Result;
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
use windows::UI::Notifications::ToastNotificationManager;

pub fn clear_all_toasts(_app_id: &str) -> Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
    }

    struct ComUninitializer;
    impl Drop for ComUninitializer {
        fn drop(&mut self) {
            unsafe {
                CoUninitialize();
            }
        }
    }
    let _com_guard = ComUninitializer;

    let history = ToastNotificationManager::History()?;

    history.Clear()?;

    Ok(())
}
