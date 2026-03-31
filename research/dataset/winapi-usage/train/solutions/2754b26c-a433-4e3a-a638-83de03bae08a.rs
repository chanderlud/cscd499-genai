use windows::core::{Error, Result};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::{AccSetRunningUtilityState, ACC_UTILITY_STATE_FLAGS};

fn call_acc_set_running_utility_state() -> Result<Result<()>> {
    let hwnd = HWND::default();
    let mask = 0u32;
    let state = ACC_UTILITY_STATE_FLAGS(0);
    // SAFETY: Calling AccSetRunningUtilityState with valid default parameters.
    let result = unsafe { AccSetRunningUtilityState(hwnd, mask, state) };
    Ok(result)
}
