use std::{collections::HashMap, ffi::OsString, os::windows::ffi::OsStringExt};
use windows::Win32::{
    Devices::Display::{
        DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME, DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
        DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_SOURCE_DEVICE_NAME,
        DISPLAYCONFIG_TARGET_DEVICE_NAME, DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes,
        QDC_ONLY_ACTIVE_PATHS, QDC_VIRTUAL_MODE_AWARE, QueryDisplayConfig,
    },
    Foundation::WIN32_ERROR,
};

fn wide_to_string(mut wide: &[u16]) -> String {
    if let Some(null) = wide.iter().position(|c| *c == 0) {
        wide = &wide[..null];
    }
    let os_string = OsString::from_wide(wide);
    os_string.to_string_lossy().into()
}

fn generate_gdi_to_monitor_name_lookup() -> windows::core::Result<HashMap<String, String>> {
    let mut paths: Vec<DISPLAYCONFIG_PATH_INFO> = Vec::new();
    let mut modes: Vec<DISPLAYCONFIG_MODE_INFO> = Vec::new();
    let flags = QDC_ONLY_ACTIVE_PATHS | QDC_VIRTUAL_MODE_AWARE;

    let mut path_count: u32 = 0;
    let mut mode_count: u32 = 0;

    // These already return WIN32_ERROR in windows-rs.
    unsafe {
        GetDisplayConfigBufferSizes(flags, &mut path_count, &mut mode_count).ok()?;
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
            .ok()?;
    }

    paths.truncate(path_count as usize);
    modes.truncate(mode_count as usize);

    let mut result = HashMap::new();
    for path in &paths {
        let mut target_name = DISPLAYCONFIG_TARGET_DEVICE_NAME::default();
        target_name.header.adapterId = path.targetInfo.adapterId;
        target_name.header.id = path.targetInfo.id;
        target_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;
        target_name.header.size = std::mem::size_of_val(&target_name) as u32;

        unsafe {
            WIN32_ERROR(DisplayConfigGetDeviceInfo(&mut target_name.header) as u32).ok()?;
        }

        let mut source_name = DISPLAYCONFIG_SOURCE_DEVICE_NAME::default();
        source_name.header.adapterId = path.sourceInfo.adapterId;
        source_name.header.id = path.sourceInfo.id;
        source_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
        source_name.header.size = std::mem::size_of_val(&source_name) as u32;

        unsafe {
            WIN32_ERROR(DisplayConfigGetDeviceInfo(&mut source_name.header) as u32).ok()?;
        }

        let monitor_name = wide_to_string(&target_name.monitorFriendlyDeviceName);
        let gdi_name = wide_to_string(&source_name.viewGdiDeviceName);

        result.insert(gdi_name, monitor_name);
    }

    Ok(result)
}

fn main() -> windows::core::Result<()> {
    let lookup = generate_gdi_to_monitor_name_lookup()?;

    println!("GDI Device Name -> Monitor Name:");
    for (gdi_name, monitor_name) in &lookup {
        println!("  {} -> {}", gdi_name, monitor_name);
    }

    Ok(())
}
