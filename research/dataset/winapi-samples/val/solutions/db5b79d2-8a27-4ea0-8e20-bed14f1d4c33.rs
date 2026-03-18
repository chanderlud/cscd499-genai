use windows::{
    core::{Error, Result, PCWSTR},
    Win32::{
        Foundation::{FALSE, HWND, LPARAM, LRESULT, WPARAM},
        UI::{
            Shell::{
                Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE,
                NOTIFYICONDATAW, NOTIFY_ICON_DATA_FLAGS, NOTIFY_ICON_MESSAGE,
            },
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, LoadIconW,
                PostQuitMessage, RegisterClassW, TranslateMessage, HICON, WM_DESTROY, WM_USER,
                WNDCLASSW, WS_OVERLAPPEDWINDOW,
            },
        },
    },
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

struct NotifyIcon {
    data: NOTIFYICONDATAW,
}

impl Default for NotifyIcon {
    fn default() -> Self {
        Self {
            data: NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as _,
                ..Default::default()
            },
        }
    }
}

impl NotifyIcon {
    fn new() -> Self {
        Self::default()
    }

    fn flag(mut self, flag: NOTIFY_ICON_DATA_FLAGS) -> Self {
        self.data.uFlags |= flag;
        self
    }

    fn window_handle(mut self, handle: HWND) -> Self {
        self.data.hWnd = handle;
        self.flag(NIF_MESSAGE)
    }

    fn tip(mut self, s: impl Into<String>) -> Self {
        let s = s.into();
        let tip_utf16 = s.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
        let max_len = self.data.szTip.len() - 1;
        if tip_utf16.len() <= max_len + 1 {
            self.data.szTip[..tip_utf16.len()].copy_from_slice(&tip_utf16);
        } else {
            self.data.szTip[..max_len].copy_from_slice(&tip_utf16[..max_len]);
            self.data.szTip[max_len] = 0;
        }
        self.flag(NIF_TIP)
    }

    fn icon(mut self, icon: HICON) -> Self {
        self.data.hIcon = icon;
        self.flag(NIF_ICON)
    }

    fn callback_message(mut self, callback_msg: u32) -> Self {
        self.data.uCallbackMessage = callback_msg;
        self.flag(NIF_MESSAGE)
    }

    fn notify(&self, message: NOTIFY_ICON_MESSAGE) -> Result<()> {
        // SAFETY: Shell_NotifyIconW is a valid Windows API call with proper parameters
        let success = unsafe { Shell_NotifyIconW(message, &self.data) };
        if success == FALSE {
            Err(Error::from_thread())
        } else {
            Ok(())
        }
    }

    fn notify_add(&self) -> Result<()> {
        self.notify(NIM_ADD)
    }

    fn notify_delete(&self) -> Result<()> {
        self.notify(NIM_DELETE)
    }
}

const TRAY_ICON_MSG: u32 = WM_USER + 1;

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        TRAY_ICON_MSG => {
            // Handle tray icon messages here
            println!("Tray icon interaction received");
            LRESULT(0)
        }
        WM_DESTROY => {
            // SAFETY: PostQuitMessage is a valid Windows API call
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => {
            // SAFETY: DefWindowProcW is a valid Windows API call with proper parameters
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
    }
}

fn main() -> Result<()> {
    // Register window class
    let class_name = wide_null("TrayIconExample".as_ref());
    let wnd_class = WNDCLASSW {
        lpfnWndProc: Some(wndproc),
        hInstance: Default::default(),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        ..Default::default()
    };

    // SAFETY: RegisterClassW is a valid Windows API call with proper parameters
    let atom = unsafe { RegisterClassW(&wnd_class) };
    if atom == 0 {
        return Err(Error::from_thread());
    }

    // Create window
    let window_name = wide_null("Tray Icon Example".as_ref());
    // SAFETY: CreateWindowExW is a valid Windows API call with proper parameters
    let hwnd = unsafe {
        CreateWindowExW(
            Default::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_name.as_ptr()),
            WS_OVERLAPPEDWINDOW,
            100,
            100,
            400,
            300,
            None,
            None,
            None,
            None,
        )
    }?;

    // Load default application icon
    // SAFETY: LoadIconW with null instance and IDI_APPLICATION is valid
    let icon = unsafe { LoadIconW(None, PCWSTR(32512 as *const u16)) }?;

    // Create and configure notification icon
    let tray_icon = NotifyIcon::new()
        .window_handle(hwnd)
        .tip("My Application")
        .icon(icon)
        .callback_message(TRAY_ICON_MSG);

    // Add icon to system tray
    tray_icon.notify_add()?;

    println!("Tray icon added. Check your system tray!");
    println!("Close the console window to remove the tray icon.");

    // Message loop
    // SAFETY: GetMessageW, TranslateMessage, DispatchMessageW are valid Windows API calls
    unsafe {
        let mut msg = Default::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    // Clean up: remove tray icon
    tray_icon.notify_delete()?;

    Ok(())
}
