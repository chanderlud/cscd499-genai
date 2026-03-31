use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::{AccSetRunningUtilityState, ACC_UTILITY_STATE_FLAGS};

fn call_acc_set_running_utility_state() -> HRESULT {
    unsafe {
        match AccSetRunningUtilityState(HWND::default(), 0, ACC_UTILITY_STATE_FLAGS(0)) {
            Ok(()) => HRESULT::default(),
            Err(e) => e.code(),
        }
    }
}
