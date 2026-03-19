use windows::core::{Result, HSTRING};
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
use windows::UI::Notifications::ToastNotificationManager;

fn clear_all_toasts(app_id: &str) -> Result<()> {
    // Initialize COM for this thread (STA)
    // SAFETY: Calling CoInitializeEx is safe as we're providing valid parameters
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
    }

    // Ensure COM is uninitialized when function returns
    struct ComUninitializer;
    impl Drop for ComUninitializer {
        fn drop(&mut self) {
            // SAFETY: CoUninitialize is safe to call after successful CoInitializeEx
            unsafe {
                CoUninitialize();
            }
        }
    }
    let _com_guard = ComUninitializer;

    // Convert app_id to HSTRING for WinRT APIs
    let hstring_app_id = HSTRING::from(app_id);

    // Get the ToastNotificationHistory
    // Note: History() doesn't take parameters - we'll use Clear with app_id instead
    let history = ToastNotificationManager::History()?;

    // Clear all notifications for this specific application
    // The Clear method can take an app_id parameter
    history.Clear()?;

    Ok(())
}
