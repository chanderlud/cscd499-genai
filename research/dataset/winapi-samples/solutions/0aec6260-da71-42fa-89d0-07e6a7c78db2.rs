use std::{collections::VecDeque, mem};
use windows::{
    core::{Result, BOOL, PCWSTR},
    Win32::{
        Foundation::{LPARAM, RECT},
        Graphics::Gdi::{
            EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW,
        },
    },
};

fn wchar_ptr_to_string(ptr: PCWSTR) -> String {
    unsafe {
        let mut len = 0;
        while *ptr.0.add(len) != 0 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(ptr.0, len);
        String::from_utf16_lossy(slice)
    }
}

#[derive(Debug, Clone)]
struct MonitorHandle(isize);

impl MonitorHandle {
    fn new(hmonitor: HMONITOR) -> Self {
        MonitorHandle(hmonitor.0 as _)
    }

    fn hmonitor(&self) -> HMONITOR {
        HMONITOR(self.0 as _)
    }

    fn name(&self) -> Result<String> {
        let monitor_info = get_monitor_info(self.hmonitor())?;
        Ok(wchar_ptr_to_string(PCWSTR::from_raw(
            monitor_info.szDevice.as_ptr(),
        )))
    }

    fn size(&self) -> (i32, i32) {
        let Ok(monitor_info) = get_monitor_info(self.hmonitor()) else {
            return (0, 0);
        };
        let rect = monitor_info.monitorInfo.rcMonitor;
        (rect.right - rect.left, rect.bottom - rect.top)
    }
}

fn get_monitor_info(hmonitor: HMONITOR) -> Result<MONITORINFOEXW> {
    let mut monitor_info = MONITORINFOEXW::default();
    monitor_info.monitorInfo.cbSize = mem::size_of::<MONITORINFOEXW>() as u32;
    unsafe {
        GetMonitorInfoW(
            hmonitor,
            &mut monitor_info as *mut MONITORINFOEXW as *mut MONITORINFO,
        )
        .ok()?;
    }
    Ok(monitor_info)
}

unsafe extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _place: *mut RECT,
    data: LPARAM,
) -> BOOL {
    let monitors = data.0 as *mut VecDeque<MonitorHandle>;
    (*monitors).push_back(MonitorHandle::new(hmonitor));
    true.into()
}

fn available_monitors() -> Result<VecDeque<MonitorHandle>> {
    let mut monitors: VecDeque<MonitorHandle> = VecDeque::new();
    unsafe {
        EnumDisplayMonitors(
            None,
            None,
            Some(monitor_enum_proc),
            LPARAM(&mut monitors as *mut _ as _),
        )
        .ok()?;
    }
    Ok(monitors)
}

fn main() -> Result<()> {
    let monitors = available_monitors()?;

    println!("Found {} monitor(s):", monitors.len());
    for (i, monitor) in monitors.iter().enumerate() {
        let name = monitor.name().unwrap_or_else(|_| "Unknown".to_string());
        let (width, height) = monitor.size();
        println!("Monitor {}: {} ({}x{})", i + 1, name, width, height);
    }

    Ok(())
}
