use windows::Win32::Foundation::{HWND, WIN32_ERROR};
use windows::Win32::UI::Accessibility::{AccSetRunningUtilityState, ACC_UTILITY_STATE_FLAGS};

fn call_acc_set_running_utility_state() -> WIN32_ERROR {
    let result =
        unsafe { AccSetRunningUtilityState(HWND::default(), 0, ACC_UTILITY_STATE_FLAGS(0)) };
    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or_default(),
    }
}
