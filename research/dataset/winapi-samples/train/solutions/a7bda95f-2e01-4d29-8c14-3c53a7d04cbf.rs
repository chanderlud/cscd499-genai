use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromPoint, MONITORINFOEXW, MONITOR_DEFAULTTOPRIMARY,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{VK_DOWN, VK_ESCAPE, VK_RETURN, VK_UP};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, GetWindowLongPtrW,
    PostQuitMessage, RegisterClassExW, SetWindowLongPtrW, ShowWindow, TranslateMessage,
    CREATESTRUCTW, CW_USEDEFAULT, GWL_USERDATA, GWL_WNDPROC, HMENU, LBS_NOINTEGRALHEIGHT,
    LBS_NOTIFY, LB_ADDSTRING, LB_GETCURSEL, LB_GETTEXT, LB_GETTEXTLEN, MSG, SW_SHOW,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_CREATE, WM_DESTROY, WM_KEYDOWN, WM_KEYUP, WNDCLASSEXW,
    WS_CHILD, WS_OVERLAPPEDWINDOW, WS_VISIBLE, WS_VSCROLL,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => {
                let create_struct = &*(lparam.0 as *const CREATESTRUCTW);
                let list_box_class = wide_null("ListBox".as_ref());
                let style = WS_CHILD
                    | WS_VISIBLE
                    | WS_VSCROLL
                    | WINDOW_STYLE(LBS_NOTIFY as u32)
                    | WINDOW_STYLE(LBS_NOINTEGRALHEIGHT as u32);

                let list_box_hwnd = CreateWindowExW(
                    WINDOW_EX_STYLE(0),
                    PCWSTR(list_box_class.as_ptr()),
                    PCWSTR(std::ptr::null()),
                    style,
                    0,
                    0,
                    0,
                    0,
                    Some(hwnd),
                    Some(HMENU(std::ptr::null_mut())),
                    Some(create_struct.hInstance),
                    None,
                )
                .unwrap();

                // Get primary monitor info
                let monitor = MonitorFromPoint(
                    windows::Win32::Foundation::POINT { x: 0, y: 0 },
                    MONITOR_DEFAULTTOPRIMARY,
                );
                let mut monitor_info = MONITORINFOEXW::default();
                monitor_info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
                GetMonitorInfoW(monitor, &mut monitor_info.monitorInfo).unwrap();

                // Format primary monitor string
                let device_name = String::from_utf16_lossy(
                    &monitor_info.szDevice[..monitor_info
                        .szDevice
                        .iter()
                        .position(|&x| x == 0)
                        .unwrap_or(0)],
                );
                let width = monitor_info.monitorInfo.rcMonitor.right
                    - monitor_info.monitorInfo.rcMonitor.left;
                let height = monitor_info.monitorInfo.rcMonitor.bottom
                    - monitor_info.monitorInfo.rcMonitor.top;
                let dpi = 96; // Default DPI
                let primary_info = format!(
                    "Primary: {}, {}x{}, DPI:{}",
                    device_name, width, height, dpi
                );

                // Add primary monitor info as first item
                let wide_primary = wide_null(primary_info.as_ref());
                SendMessageW(
                    list_box_hwnd,
                    LB_ADDSTRING,
                    None,
                    Some(lparam_from_slice(&wide_primary)),
                );

                // Add nine additional items
                for i in 1..=9 {
                    let item = format!("Item {}", i);
                    let wide_item = wide_null(item.as_ref());
                    SendMessageW(
                        list_box_hwnd,
                        LB_ADDSTRING,
                        None,
                        Some(lparam_from_slice(&wide_item)),
                    );
                }

                // Subclass the ListBox
                let old_proc =
                    SetWindowLongPtrW(list_box_hwnd, GWL_WNDPROC, list_box_proc as usize as isize);
                SetWindowLongPtrW(list_box_hwnd, GWL_USERDATA, old_proc);

                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

extern "system" fn list_box_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        let old_proc = GetWindowLongPtrW(hwnd, GWL_USERDATA) as usize;
        let old_proc: extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT =
            std::mem::transmute(old_proc);

        match msg {
            WM_KEYDOWN => {
                // Extract virtual key values as constants for pattern matching
                const ESCAPE: u16 = VK_ESCAPE.0;
                const RETURN: u16 = VK_RETURN.0;
                const UP: u16 = VK_UP.0;
                const DOWN: u16 = VK_DOWN.0;

                match wparam.0 as u16 {
                    ESCAPE => {
                        PostQuitMessage(0);
                        LRESULT(0)
                    }
                    RETURN => {
                        let sel = SendMessageW(hwnd, LB_GETCURSEL, None, None);
                        if sel.0 >= 0 {
                            let len = SendMessageW(
                                hwnd,
                                LB_GETTEXTLEN,
                                Some(WPARAM(sel.0 as usize)),
                                None,
                            );
                            let mut buf = vec![0u16; (len.0 as usize) + 1];
                            SendMessageW(
                                hwnd,
                                LB_GETTEXT,
                                Some(WPARAM(sel.0 as usize)),
                                Some(LPARAM(buf.as_ptr() as isize)),
                            );
                            let text = String::from_utf16_lossy(&buf[..len.0 as usize]);
                            println!("Selected: {}", text);
                        }
                        LRESULT(0)
                    }
                    UP | DOWN => old_proc(hwnd, msg, wparam, lparam),
                    _ => old_proc(hwnd, msg, wparam, lparam),
                }
            }
            WM_KEYUP => {
                const UP: u16 = VK_UP.0;
                const DOWN: u16 = VK_DOWN.0;

                match wparam.0 as u16 {
                    UP | DOWN => old_proc(hwnd, msg, wparam, lparam),
                    _ => LRESULT(0),
                }
            }
            _ => old_proc(hwnd, msg, wparam, lparam),
        }
    }
}

fn lparam_from_slice(v: &[u16]) -> LPARAM {
    LPARAM(v.as_ptr() as isize)
}

fn SendMessageW(hwnd: HWND, msg: u32, wparam: Option<WPARAM>, lparam: Option<LPARAM>) -> LRESULT {
    unsafe { windows::Win32::UI::WindowsAndMessaging::SendMessageW(hwnd, msg, wparam, lparam) }
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class_name = wide_null("MonitorListBoxClass".as_ref());

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(window_proc),
            hInstance: instance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        RegisterClassExW(&wc);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(wide_null("Monitor Info ListBox".as_ref()).as_ptr()),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        ShowWindow(hwnd, SW_SHOW);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Ok(())
    }
}
