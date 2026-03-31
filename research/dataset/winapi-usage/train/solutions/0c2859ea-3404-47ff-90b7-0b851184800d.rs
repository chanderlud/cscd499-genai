use windows::core::{Error, Result};
use windows::Win32::System::Environment::ExpandEnvironmentStringsW;

fn call_expand_environment_strings_w() -> windows::core::HRESULT {
    unsafe { ExpandEnvironmentStringsW(windows::core::w!("test"), None) };
    windows::core::HRESULT::from_win32(0)
}
