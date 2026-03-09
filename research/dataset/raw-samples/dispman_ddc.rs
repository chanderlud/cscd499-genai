use crate::error::DisplayError;
use windows::Win32::Devices::Display::{
    GetCapabilitiesStringLength, GetVCPFeatureAndVCPFeatureReply, SetVCPFeature, CapabilitiesRequestAndCapabilitiesReply,
};
use windows::Win32::Foundation::HANDLE;

pub fn get_vcp_feature(handle: HANDLE, code: u8) -> Result<u32, DisplayError> {
    let mut current_value: u32 = 0;
    let mut max_value: u32 = 0;
    let success: i32;

    unsafe {
        success = GetVCPFeatureAndVCPFeatureReply(
            handle,
            code,
            None,
            &mut current_value,
            Some(&mut max_value),
        );
    }

    if success != 0 {
        Ok(current_value)
    } else {
        Err(DisplayError::DdcCommunicationFailed)
    }
}

pub fn set_vcp_feature(handle: HANDLE, code: u8, value: u32) -> Result<(), DisplayError> {
    let success: i32;
    unsafe {
        success = SetVCPFeature(handle, code, value);
    }

    if success != 0 {
        Ok(())
    } else {
        Err(DisplayError::DdcCommunicationFailed)
    }
}

pub fn get_capabilities(handle: HANDLE) -> Result<String, DisplayError> {
    let mut length: u32 = 0;
    let success: i32;
    unsafe {
        success = GetCapabilitiesStringLength(handle, &mut length);
    }

    if success == 0 {
        return Err(DisplayError::DdcCommunicationFailed);
    }

    let mut buffer = vec![0u8; length as usize];
    let success_str: i32;
    unsafe {
        success_str = CapabilitiesRequestAndCapabilitiesReply(handle, &mut buffer);
    }

    if success_str == 0 {
        return Err(DisplayError::DdcCommunicationFailed);
    }

    // Convert to string and trim null bytes
    let s = String::from_utf8_lossy(&buffer).to_string();
    Ok(s.trim_matches(char::from(0)).to_string())
}