use windows::core::{Error, Result, HRESULT, PWSTR};
use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, ERROR_NO_TOKEN, HANDLE};
use windows::Win32::Security::{
    GetTokenInformation, LookupPrivilegeNameW, TokenPrivileges, LUID_AND_ATTRIBUTES,
    TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::Win32::UI::Input::KeyboardAndMouse::{MapVirtualKeyExW, HKL, MAPVK_VSC_TO_VK_EX};

#[derive(Debug, PartialEq)]
enum KeyLocation {
    Standard,
    Left,
    Right,
    Numpad,
}

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

fn has_privilege(required_privilege: &str) -> Result<bool> {
    // SAFETY: GetCurrentProcess returns a pseudo handle that's always valid
    let process_handle = unsafe { GetCurrentProcess() };
    let mut token_handle = HANDLE::default();

    // SAFETY: We're passing a valid process handle and a mutable pointer to receive the token
    unsafe {
        OpenProcessToken(process_handle, TOKEN_QUERY, &mut token_handle)
            .map_err(|_| Error::from_thread())?;
    }

    // First call to get required buffer size
    let mut buffer_size = 0u32;
    // SAFETY: We're passing a null buffer to get the required size
    unsafe {
        GetTokenInformation(token_handle, TokenPrivileges, None, 0, &mut buffer_size);
    }

    if buffer_size == 0 {
        let error = unsafe { Error::from_thread() };
        if error.code() != HRESULT::from_win32(ERROR_INSUFFICIENT_BUFFER.0) {
            return Err(error);
        }
    }

    // Allocate buffer with the required size
    let mut buffer = vec![0u8; buffer_size as usize];
    // SAFETY: We're passing a properly sized buffer and valid token handle
    unsafe {
        GetTokenInformation(
            token_handle,
            TokenPrivileges,
            Some(buffer.as_mut_ptr() as *mut _),
            buffer_size,
            &mut buffer_size,
        )
        .map_err(|_| Error::from_thread())?;
    }

    // SAFETY: The buffer is properly initialized by GetTokenInformation
    let token_privileges = unsafe { &*(buffer.as_ptr() as *const TOKEN_PRIVILEGES) };
    let privilege_count = token_privileges.PrivilegeCount as usize;

    // SAFETY: We're calculating the offset to the LUID_AND_ATTRIBUTES array
    let privileges_ptr = unsafe {
        buffer
            .as_ptr()
            .add(std::mem::offset_of!(TOKEN_PRIVILEGES, Privileges))
    };

    // SAFETY: We're creating a slice from the pointer and count
    let privileges = unsafe {
        std::slice::from_raw_parts(
            privileges_ptr as *const LUID_AND_ATTRIBUTES,
            privilege_count,
        )
    };

    for privilege in privileges {
        let mut name_buffer = [0u16; 256];
        let mut name_length = name_buffer.len() as u32;

        // SAFETY: We're passing valid LUID and buffer to lookup privilege name
        unsafe {
            LookupPrivilegeNameW(
                None,
                &privilege.Luid,
                Some(PWSTR(name_buffer.as_mut_ptr())),
                &mut name_length,
            )
            .map_err(|_| Error::from_thread())?;
        }

        let privilege_name = String::from_utf16_lossy(&name_buffer[..name_length as usize]);
        if privilege_name == required_privilege {
            return Ok(true);
        }
    }

    Ok(false)
}

fn get_key_location_if_privileged(
    scancode: u16,
    hkl: HKL,
    required_privilege: &str,
) -> Result<KeyLocation> {
    if !has_privilege(required_privilege)? {
        return Err(Error::new(
            HRESULT::from_win32(ERROR_NO_TOKEN.0),
            "Required privilege not held by process",
        ));
    }

    // Convert scancode to virtual key using the provided keyboard layout
    // SAFETY: We're passing a valid scancode and HKL
    let vk = unsafe { MapVirtualKeyExW(scancode as u32, MAPVK_VSC_TO_VK_EX, Some(hkl)) };

    if vk == 0 {
        return Err(Error::from_thread());
    }

    // Determine key location based on virtual key code
    let location = match vk as u16 {
        // Modifier keys with left/right variants
        0x10 | 0x11 | 0x12 => {
            // VK_SHIFT, VK_CONTROL, VK_MENU
            // Check if this is an extended key (right side)
            if scancode & 0xE000 != 0 {
                KeyLocation::Right
            } else {
                KeyLocation::Left
            }
        }

        // Navigation keys with numpad variants
        0x21 | 0x22 | 0x23 | 0x24 | 0x25 | 0x26 | 0x27 | 0x28 | 0x2D | 0x2E => {
            // VK_PRIOR, VK_NEXT, VK_END, VK_HOME, VK_LEFT, VK_UP, VK_RIGHT, VK_DOWN, VK_INSERT, VK_DELETE
            // Check if this is from numpad (not extended)
            if scancode & 0xE000 == 0 {
                KeyLocation::Numpad
            } else {
                KeyLocation::Standard
            }
        }

        // Enter key has numpad variant
        0x0D => {
            // VK_RETURN
            if scancode & 0xE000 == 0 {
                KeyLocation::Numpad
            } else {
                KeyLocation::Standard
            }
        }

        // All other keys are standard
        _ => KeyLocation::Standard,
    };

    Ok(location)
}
