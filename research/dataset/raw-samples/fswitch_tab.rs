use windows::Win32::{Foundation::{BOOL, HWND, LPARAM}, Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED}, UI::{Input::KeyboardAndMouse::IsWindowEnabled, WindowsAndMessaging::{EnumWindows, GetWindowLongW, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible, GWL_EXSTYLE, WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW}}};



pub struct WindowInfo {
    pub hwnd: HWND,
    pub title: Vec<u16>,
}

unsafe extern "system" fn enum_window_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let window_list: &mut Vec<WindowInfo> = &mut *(lparam.0 as *mut Vec<WindowInfo>);


    // Check if the window is visible
    if !IsWindowVisible(hwnd).as_bool() {
        return BOOL::from(true);
    }

    if !IsWindowEnabled(hwnd).as_bool() {
        return BOOL::from(true);
    }

    // check if the window is clocked (hidden)
    let mut cloaked: u32 = 0;
    let hr = DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, &mut cloaked as *mut _ as *mut _, std::mem::size_of::<u32>() as u32);
    if hr.is_ok() && cloaked != 0 {
        return BOOL::from(true);
    }

    // Check if the window is a top-level application window
    let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
    if ex_style & WS_EX_APPWINDOW.0 != 0
        || ex_style & WS_EX_NOACTIVATE.0 != 0
        || ex_style & WS_EX_TOOLWINDOW.0 != 0
    {
        return BOOL::from(true);
    }

    // Get the length of the window title
    let length = GetWindowTextLengthW(hwnd);

    // only add the window to the window list, if:
    // it has a title
    // and none of above predicates are matching...
    if length > 0 {
        // Create a buffer to hold the window title
        let mut buffer = vec![0u16; (length + 1) as usize];
        GetWindowTextW(hwnd, &mut buffer);

        window_list.push(WindowInfo { hwnd, title: buffer });
    }


    // Continue enumeration
    BOOL::from(true)
}



pub fn get_open_windows() -> Vec<WindowInfo> {
    let mut window_list: Vec<WindowInfo> = Vec::new();

    // Pass a mutable reference to the vector as LPARAM
    let lparam = LPARAM(&mut window_list as *mut _ as isize);

    unsafe {
        EnumWindows(Some(enum_window_callback), lparam).unwrap();
    }

    window_list
}