// System Tray Icon with GUID and Version Configuration

use windows::{
    core::{Error, Result, GUID, PCWSTR},
    Win32::{
        Foundation::{FALSE, HWND, LPARAM, LRESULT, WPARAM},
        UI::{
            Shell::{
                Shell_NotifyIconW, NIF_GUID, NIF_ICON, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIM_ADD,
                NIM_DELETE, NIM_SETVERSION, NOTIFYICONDATAW, NOTIFYICON_VERSION_4,
                NOTIFY_ICON_DATA_FLAGS, NOTIFY_ICON_MESSAGE,
            },
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, LoadIconW,
                PostQuitMessage, RegisterClassW, TranslateMessage, HICON, MSG, WINDOW_EX_STYLE,
                WINDOW_STYLE, WM_DESTROY, WM_USER, WNDCLASSW,
            },
        },
    },
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
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
        self.flag(NIF_TIP | NIF_SHOWTIP)
    }

    fn icon(mut self, icon: HICON) -> Self {
        self.data.hIcon = icon;
        self.flag(NIF_ICON)
    }

    fn callback_message(mut self, callback_msg: u32) -> Self {
        self.data.uCallbackMessage = callback_msg;
        self.flag(NIF_MESSAGE)
    }

    fn guid(mut self, guid: impl Into<GUID>) -> Self {
        self.data.guidItem = guid.into();
        self.flag(NIF_GUID)
    }

    fn version(mut self, version: u32) -> Self {
        self.data.Anonymous.uVersion = version;
        self
    }

    fn notify(&self, message: NOTIFY_ICON_MESSAGE) -> Result<()> {
        // SAFETY: Shell_NotifyIconW is a valid Windows API call with proper parameters
        let result = unsafe { Shell_NotifyIconW(message, &self.data) };
        if result == FALSE {
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

    fn notify_set_version(&self) -> Result<()> {
        self.notify(NIM_SETVERSION)
    }
}

const MY_CALLBACK_MSG: u32 = WM_USER + 1;

// SAFETY: Window procedure callback - must be unsafe due to Windows API requirements
unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn main() -> Result<()> {
    // Register window class
    let class_name = wide_null("MyWindowClass".as_ref());
    let wnd_class = WNDCLASSW {
        lpfnWndProc: Some(wndproc),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        ..Default::default()
    };

    // SAFETY: RegisterClassW is a valid Windows API call with proper parameters
    let atom = unsafe { RegisterClassW(&wnd_class) };
    if atom == 0 {
        return Err(Error::from_thread());
    }

    // Create window
    let window_name = wide_null("My Window".as_ref());
    // SAFETY: CreateWindowExW is a valid Windows API call with proper parameters
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_name.as_ptr()),
            WINDOW_STYLE::default(),
            0,
            0,
            0,
            0,
            None,
            None,
            None,
            None,
        )
    }?;

    // Load icon
    // SAFETY: LoadIconW with null instance loads standard system icons
    let icon = unsafe { LoadIconW(None, PCWSTR(32512 as *const u16)) }?;

    // Create GUID for persistent icon identification
    let icon_guid = GUID::from_values(
        0x12345678,
        0x1234,
        0x5678,
        [0x90, 0xAB, 0xCD, 0xEF, 0x12, 0x34, 0x56, 0x78],
    );

    // Create and configure notification icon with GUID and version
    let notify_icon = NotifyIcon::new()
        .window_handle(hwnd)
        .tip("My Application with GUID")
        .icon(icon)
        .callback_message(MY_CALLBACK_MSG)
        .guid(icon_guid)
        .version(NOTIFYICON_VERSION_4);

    // Add icon to system tray and set version
    notify_icon.notify_add()?;
    notify_icon.notify_set_version()?;

    // Message loop
    let mut msg = MSG::default();
    // SAFETY: GetMessageW is a valid Windows API call with proper parameters
    while unsafe { GetMessageW(&mut msg, None, 0, 0) }.into() {
        let _ = unsafe { TranslateMessage(&msg) };
        unsafe {
            DispatchMessageW(&msg);
        }
    }

    // Clean up - remove icon before exit
    notify_icon.notify_delete()?;

    Ok(())
}
