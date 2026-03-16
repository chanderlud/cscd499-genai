use windows::core::{Result, Error, HRESULT, PCWSTR};
use windows::Win32::Foundation::{HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT, ERROR_SUCCESS, CloseHandle};
use windows::Win32::System::Registry::{
    RegOpenKeyExW, RegCloseKey, RegNotifyChangeKeyValue, HKEY, HKEY_CURRENT_USER,
    REG_NOTIFY_CHANGE_NAME, REG_NOTIFY_CHANGE_LAST_SET, REG_OPTION_OPEN_LINK,
    KEY_READ,
};
use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn wait_for_reg_change_hkcu(path: &str, timeout_ms: u32) -> Result<bool> {
    let wide_path = wide_null(std::ffi::OsStr::new(path));
    
    // Open the registry key
    let mut hkey = HKEY::default();
    let result = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(wide_path.as_ptr()),
            Some(REG_OPTION_OPEN_LINK.0),
            KEY_READ,
            &mut hkey,
        )
    };
    
    if result != ERROR_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_win32(result.0)));
    }
    
    // Ensure we close the key on any exit path
    let key_guard = RegistryKeyGuard(hkey);
    
    // Create an event for notification
    let event = unsafe { CreateEventW(None, true, false, None) }?;
    if event.is_invalid() {
        return Err(Error::from_thread());
    }
    
    // Ensure we close the event on any exit path
    let event_guard = EventGuard(event);
    
    // Request notification for both name and value changes
    let filter = REG_NOTIFY_CHANGE_NAME | REG_NOTIFY_CHANGE_LAST_SET;
    
    let result = unsafe {
        RegNotifyChangeKeyValue(
            key_guard.0,
            true,
            filter,
            Some(event_guard.0),
            true,
        )
    };
    
    if result != ERROR_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_win32(result.0)));
    }
    
    // Wait for the event with timeout
    let wait_result = unsafe { WaitForSingleObject(event_guard.0, timeout_ms) };
    
    match wait_result {
        WAIT_OBJECT_0 => Ok(true),
        WAIT_TIMEOUT => Ok(false),
        _ => Err(Error::from_thread()),
    }
}

// RAII guards for automatic cleanup
struct RegistryKeyGuard(HKEY);
impl Drop for RegistryKeyGuard {
    fn drop(&mut self) {
        unsafe { RegCloseKey(self.0) };
    }
}

struct EventGuard(HANDLE);
impl Drop for EventGuard {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
    }
}