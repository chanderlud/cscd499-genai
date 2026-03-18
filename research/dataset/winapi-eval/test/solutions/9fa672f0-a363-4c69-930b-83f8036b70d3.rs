use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::core::PCWSTR;
use windows::Win32::System::Environment::{FreeEnvironmentStringsW, GetEnvironmentStringsW};

pub fn get_environment_kv() -> std::io::Result<Vec<(String, String)>> {
    // Get the environment strings block
    let env_block = unsafe { GetEnvironmentStringsW() };
    if env_block.is_null() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to get environment strings",
        ));
    }

    // Parse the environment block
    let result = parse_environment_block(env_block);

    // Free the environment block
    unsafe {
        FreeEnvironmentStringsW(PCWSTR::from_raw(env_block.as_ptr()))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    };

    result
}

fn parse_environment_block(
    env_block: windows::core::PWSTR,
) -> std::io::Result<Vec<(String, String)>> {
    let mut result = Vec::new();

    if env_block.is_null() {
        return Ok(result);
    }

    let mut current = env_block.as_ptr();

    unsafe {
        loop {
            // Find the length of the current string (until null terminator)
            let mut len = 0;
            while *current.add(len) != 0 {
                len += 1;
            }

            // If length is 0, we've reached the double-null terminator
            if len == 0 {
                break;
            }

            // Convert the UTF-16 string to Rust String
            let slice = std::slice::from_raw_parts(current, len);
            let os_string = OsString::from_wide(slice);
            let entry = os_string.to_string_lossy().into_owned();

            // Parse key=value pairs
            if let Some(pos) = entry.find('=') {
                let key = &entry[..pos];
                let value = &entry[pos + 1..];

                // Only include entries with non-empty keys
                if !key.is_empty() {
                    result.push((key.to_string(), value.to_string()));
                }
            }

            // Move to the next string (skip the null terminator)
            current = current.add(len + 1);
        }
    }

    Ok(result)
}
