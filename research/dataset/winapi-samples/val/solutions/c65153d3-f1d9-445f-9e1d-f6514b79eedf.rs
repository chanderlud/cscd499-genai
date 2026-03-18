use std::{collections::BTreeSet, mem};
use windows::{
    core::{Error, Result, PCWSTR},
    Win32::{Foundation::POINT, Graphics::Gdi::*},
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct VideoMode {
    size: (u32, u32),
    bit_depth: u16,
    refresh_rate: u16,
}

impl Ord for VideoMode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.size
            .cmp(&other.size)
            .then(self.bit_depth.cmp(&other.bit_depth))
            .then(self.refresh_rate.cmp(&other.refresh_rate))
    }
}

impl PartialOrd for VideoMode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn get_monitor_info(hmonitor: HMONITOR) -> Result<MONITORINFOEXW> {
    let mut monitor_info = MONITORINFOEXW::default();
    monitor_info.monitorInfo.cbSize = mem::size_of::<MONITORINFOEXW>() as u32;

    // SAFETY: GetMonitorInfoW is a valid Win32 API call with properly initialized struct
    let status = unsafe {
        GetMonitorInfoW(
            hmonitor,
            &mut monitor_info as *mut MONITORINFOEXW as *mut MONITORINFO,
        )
    };

    if !status.as_bool() {
        Err(Error::from_thread())
    } else {
        Ok(monitor_info)
    }
}

fn primary_monitor() -> HMONITOR {
    const ORIGIN: POINT = POINT { x: 0, y: 0 };
    // SAFETY: MonitorFromPoint is a valid Win32 API call with valid parameters
    unsafe { MonitorFromPoint(ORIGIN, MONITOR_DEFAULTTOPRIMARY) }
}

fn enumerate_video_modes(hmonitor: HMONITOR) -> Result<BTreeSet<VideoMode>> {
    let monitor_info = get_monitor_info(hmonitor)?;
    let device_name = PCWSTR::from_raw(monitor_info.szDevice.as_ptr());
    let mut modes = BTreeSet::new();
    let mut i = 0;

    loop {
        // SAFETY: EnumDisplaySettingsExW is a valid Win32 API call with properly initialized DEVMODEW
        unsafe {
            let mut mode: DEVMODEW = mem::zeroed();
            mode.dmSize = mem::size_of_val(&mode) as u16;

            if !EnumDisplaySettingsExW(
                device_name,
                ENUM_DISPLAY_SETTINGS_MODE(i),
                &mut mode,
                ENUM_DISPLAY_SETTINGS_FLAGS(0),
            )
            .as_bool()
            {
                break;
            }

            i += 1;

            let required_fields =
                DM_BITSPERPEL | DM_PELSWIDTH | DM_PELSHEIGHT | DM_DISPLAYFREQUENCY;
            if mode.dmFields & required_fields == required_fields {
                modes.insert(VideoMode {
                    size: (mode.dmPelsWidth, mode.dmPelsHeight),
                    bit_depth: mode.dmBitsPerPel as u16,
                    refresh_rate: mode.dmDisplayFrequency as u16,
                });
            }
        }
    }

    Ok(modes)
}

fn main() -> Result<()> {
    let hmonitor = primary_monitor();
    let video_modes = enumerate_video_modes(hmonitor)?;

    println!("Primary Monitor Video Modes:");
    for (i, mode) in video_modes.iter().enumerate() {
        println!(
            "  Mode {}: {}x{} @ {}Hz, {} bit",
            i + 1,
            mode.size.0,
            mode.size.1,
            mode.refresh_rate,
            mode.bit_depth
        );
    }

    Ok(())
}
