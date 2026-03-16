use windows::core::{Result, Error};
use windows::Win32::System::Environment::{GetEnvironmentStringsW, FreeEnvironmentStringsW};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

pub fn get_environment_kv() -> std::io::Result<Vec<(String, String)>> {
    // Get the environment strings block
    let env_block = unsafe { GetEnvironmentStringsW()? };
    
    // Parse the environment block and ensure it's freed even if parsing fails
    let result = parse_environment_block(env_block);
    
    // Free the environment block
    unsafe { FreeEnvironmentStringsW(env_block)? };
    
    result
}

fn parse_environment_block(env_block: *const u16) -> std::io::Result<Vec<(String, String)>> {
    let mut result = Vec::new();
    
    if env_block.is_null() {
        return Ok(result);
    }
    
    let mut current = env_block;
    
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