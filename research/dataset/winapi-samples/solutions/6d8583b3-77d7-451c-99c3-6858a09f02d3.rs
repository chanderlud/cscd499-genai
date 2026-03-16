use std::{
    collections::HashMap,
    ffi::{c_void, OsString},
    os::windows::ffi::OsStringExt,
};
use windows::core::{Result, HRESULT};
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

fn wide_to_string(mut wide: &[u16]) -> String {
    if let Some(null) = wide.iter().position(|c| *c == 0) {
        wide = &wide[..null];
    }
    let os_string = OsString::from_wide(wide);
    os_string.to_string_lossy().into()
}

pub fn generate_gdi_to_monitor_name_lookup() -> Result<HashMap<String, String>> {
    let mut paths: Vec<DISPLAYCONFIG_PATH_INFO> = Vec::new();
    let mut modes: Vec<DISPLAYCONFIG_MODE_INFO> = Vec::new();
    let flags = QDC_ONLY_ACTIVE_PATHS | QDC_VIRTUAL_MODE_AWARE;

    let mut path_count: u32 = 0;
    let mut mode_count: u32 = 0;

    // SAFETY: Calling Win32 API with valid pointers
    unsafe {
        GetDisplayConfigBufferSizes(flags, &mut path_count, &mut mode_count).ok()?;
    }

    paths.resize(path_count as usize, DISPLAYCONFIG_PATH_INFO::default());
    modes.resize(mode_count as usize, DISPLAYCONFIG_MODE_INFO::default());

    // SAFETY: Calling Win32 API with valid pointers and properly sized buffers
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

    for path in paths.iter() {
        let mut target_name = DISPLAYCONFIG_TARGET_DEVICE_NAME::default();
        target_name.header.adapterId = path.targetInfo.adapterId;
        target_name.header.id = path.targetInfo.id;
        target_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;
        target_name.header.size = std::mem::size_of_val(&target_name) as u32;

        // SAFETY: Calling Win32 API with properly initialized structure
        let ret = unsafe { DisplayConfigGetDeviceInfo(&mut target_name.header) };
        HRESULT(ret).ok()?;

        let mut source_name = DISPLAYCONFIG_SOURCE_DEVICE_NAME::default();
        source_name.header.adapterId = path.targetInfo.adapterId;
        source_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
        source_name.header.size = std::mem::size_of_val(&source_name) as u32;

        // SAFETY: Calling Win32 API with properly initialized structure
        let ret = unsafe { DisplayConfigGetDeviceInfo(&mut source_name.header) };
        HRESULT(ret).ok()?;

        let monitor_name = wide_to_string(&target_name.monitorFriendlyDeviceName);
        let gdi_name = wide_to_string(&source_name.viewGdiDeviceName);

        result.insert(gdi_name, monitor_name);
    }

    Ok(result)
}

pub fn monitor_name_from_hmonitor(
    lookup: &mut HashMap<String, String>,
    handle: isize,
) -> Option<String> {
    let handle = HMONITOR(handle as *mut c_void);
    let mut monitor_info = MONITORINFOEXW::default();
    monitor_info.monitorInfo.cbSize = std::mem::size_of_val(&monitor_info) as u32;

    // SAFETY: Calling Win32 API with properly initialized structure
    if unsafe { GetMonitorInfoW(handle, &mut monitor_info.monitorInfo).as_bool() } {
        let gdi_name = wide_to_string(&monitor_info.szDevice);
        return lookup.remove(&gdi_name);
    }

    None
}

fn main() -> Result<()> {
    let mut lookup = generate_gdi_to_monitor_name_lookup()?;

    // Example: Get primary monitor handle (simplified for demonstration)
    // In real usage, you would obtain HMONITOR from MonitorFromWindow or similar
    let primary_monitor_handle = 1; // Placeholder - would be actual HMONITOR value

    if let Some(monitor_name) = monitor_name_from_hmonitor(&mut lookup, primary_monitor_handle) {
        println!("Monitor name: {}", monitor_name);
    } else {
        println!("Could not find monitor name for handle");
    }

    Ok(())
}
