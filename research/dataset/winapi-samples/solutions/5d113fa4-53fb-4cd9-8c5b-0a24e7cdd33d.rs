// Notify the shell of an association change

use windows::core::{Error, Result};
use windows::Win32::UI::Shell::{SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_IDLIST};

fn shell_change_notify() {
    // SAFETY: Calling SHChangeNotify with valid flags and null parameters
    // is safe for notifying the shell of system-wide changes.
    unsafe { SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None) };
}

fn main() {
    shell_change_notify();
}
