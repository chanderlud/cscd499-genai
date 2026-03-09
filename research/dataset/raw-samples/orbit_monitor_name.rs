use std::{collections::HashMap, ffi::OsString, os::windows::ffi::OsStringExt};

use windows::Win32::{
    Devices::Display::{
        DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QueryDisplayConfig,
        DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME, DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
        DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_SOURCE_DEVICE_NAME,
        DISPLAYCONFIG_TARGET_DEVICE_NAME, QDC_ONLY_ACTIVE_PATHS, QDC_VIRTUAL_MODE_AWARE,
    },
    Foundation::WIN32_ERROR,
    Graphics::Gdi::{GetMonitorInfoW, HMONITOR, MONITORINFOEXW},
};

pub fn generate_gdi_to_monitor_name_lookup() -> HashMap<String, String> {
    let mut paths: Vec<DISPLAYCONFIG_PATH_INFO> = Vec::new();
    let mut modes: Vec<DISPLAYCONFIG_MODE_INFO> = Vec::new();
    let flags = QDC_ONLY_ACTIVE_PATHS | QDC_VIRTUAL_MODE_AWARE;

    let mut path_count: u32 = 0;
    let mut mode_count: u32 = 0;
    unsafe {
        GetDisplayConfigBufferSizes(flags, &mut path_count, &mut mode_count).unwrap();
    }

    paths.resize(path_count as usize, DISPLAYCONFIG_PATH_INFO::default());
    modes.resize(mode_count as usize, DISPLAYCONFIG_MODE_INFO::default());

    unsafe {
        QueryDisplayConfig(
            flags,
            &mut path_count,
            paths.as_mut_ptr(),
            &mut mode_count,
            modes.as_mut_ptr(),
            None,
        )
            .unwrap();
    }
    paths.truncate(path_count as usize);
    modes.truncate(mode_count as usize);

    paths
        .iter()
        .map(|path| {
            let mut target_name = DISPLAYCONFIG_TARGET_DEVICE_NAME::default();
            target_name.header.adapterId = path.targetInfo.adapterId;
            target_name.header.id = path.targetInfo.id;
            target_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;
            target_name.header.size = std::mem::size_of_val(&target_name) as u32;
            WIN32_ERROR(unsafe { DisplayConfigGetDeviceInfo(&mut target_name.header) } as u32).ok().unwrap();

            let mut source_name = DISPLAYCONFIG_SOURCE_DEVICE_NAME::default();
            source_name.header.adapterId = path.targetInfo.adapterId;
            source_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
            source_name.header.size = std::mem::size_of_val(&source_name) as u32;
            WIN32_ERROR(unsafe { DisplayConfigGetDeviceInfo(&mut source_name.header) } as u32).ok().unwrap();

            let monitor_name = wide_to_string(&target_name.monitorFriendlyDeviceName);
            let gdi_name = wide_to_string(&source_name.viewGdiDeviceName);

            (gdi_name, monitor_name)
        })
        .collect()
}

fn wide_to_string(mut wide: &[u16]) -> String {
    if let Some(null) = wide.iter().position(|c| *c == 0) {
        wide = &wide[..null];
    }
    let os_string = OsString::from_wide(wide);
    os_string.to_string_lossy().into()
}

pub fn monitor_name_form_hmonitor(lookup: &mut HashMap<String, String>, handle: isize) -> Option<String> {
    let handle = HMONITOR(handle);
    let mut monitor_info = MONITORINFOEXW::default();
    monitor_info.monitorInfo.cbSize = std::mem::size_of_val(&monitor_info) as u32;
    if unsafe { GetMonitorInfoW(handle, &mut monitor_info.monitorInfo).as_bool() } {
        let gdi_name = wide_to_string(&monitor_info.szDevice);
        return lookup.remove(&gdi_name);
    }

    None
}