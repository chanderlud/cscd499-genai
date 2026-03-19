use windows::{
    core::{Error, Result, PCWSTR},
    Win32::{
        Foundation::{FALSE, HWND, LPARAM, LRESULT, WPARAM},
        UI::{
            Shell::{
                Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIM_ADD,
                NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW, NOTIFY_ICON_DATA_FLAGS,
                NOTIFY_ICON_MESSAGE,
            },
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, LoadIconW,
                PostQuitMessage, RegisterClassExW, TranslateMessage, HICON, WM_DESTROY, WM_USER,
                WNDCLASSEXW, WS_EX_OVERLAPPEDWINDOW, WS_OVERLAPPEDWINDOW,
            },
        },
    },
};

/// A wrapper around the Windows NOTIFYICONDATAW structure for managing system
/// tray icons in Windows.
pub struct NotifyIcon {
    /// Underlying internal data.
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
    /// Creates a new [NotifyIcon] instance with default values.
    pub fn new() -> NotifyIcon {
        Self::default()
    }

    /// Sets a flag in the notification icon data structure.
    pub fn flag(mut self, flag: NOTIFY_ICON_DATA_FLAGS) -> Self {
        self.data.uFlags |= flag;
        self
    }

    /// Sets the window handle that will receive notification messages.
    pub fn window_handle(mut self, handle: HWND) -> Self {
        self.data.hWnd = handle;
        self.flag(NIF_MESSAGE)
    }

    /// Sets the tooltip text for the notification icon.
    pub fn tip(mut self, s: impl Into<String>) -> Self {
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

    /// Sets the icon for the notification area.
    pub fn icon(mut self, icon: HICON) -> Self {
        self.data.hIcon = icon;
        self.flag(NIF_ICON)
    }

    /// Sets the callback message identifier for the notification icon.
    pub fn callback_message(mut self, callback_msg: u32) -> Self {
        self.data.uCallbackMessage = callback_msg;
        self.flag(NIF_MESSAGE)
    }

    /// Sends a notification message to the Windows shell.
    pub fn notify(&self, message: NOTIFY_ICON_MESSAGE) -> Result<()> {
        (unsafe { Shell_NotifyIconW(message, &self.data) } != FALSE)
            .then_some(())
            .ok_or_else(Error::from_thread)
    }

    /// Adds the notification icon to the system tray.
    pub fn notify_add(&self) -> Result<()> {
        self.notify(NIM_ADD)
    }

    /// Modifies an existing notification icon in the system tray.
    pub fn notify_modify(&self) -> Result<()> {
        self.notify(NIM_MODIFY)
    }

    /// Removes the notification icon from the system tray.
    pub fn notify_delete(&self) -> Result<()> {
        self.notify(NIM_DELETE)
    }
}

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    let class_name = wide_null(std::ffi::OsStr::new("TrayIconExample"));
    let window_name = wide_null(std::ffi::OsStr::new("Tray Icon Example"));

    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        lpfnWndProc: Some(wndproc),
        hInstance: unsafe { windows::Win32::System::LibraryLoader::GetModuleHandleW(None)?.into() },
        lpszClassName: PCWSTR(class_name.as_ptr()),
        ..Default::default()
    };

    // SAFETY: RegisterClassExW is safe to call with valid parameters
    let atom = unsafe { RegisterClassExW(&wc) };
    if atom == 0 {
        return Err(Error::from_thread());
    }

    // SAFETY: CreateWindowExW is safe to call with valid parameters
    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_OVERLAPPEDWINDOW,
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_name.as_ptr()),
            WS_OVERLAPPEDWINDOW,
            100,
            100,
            400,
            300,
            None,
            None,
            Some(windows::Win32::System::LibraryLoader::GetModuleHandleW(None)?.into()),
            None,
        )
    }?;

    // SAFETY: LoadIconW is safe to call with valid parameters
    let icon = unsafe {
        LoadIconW(
            None,
            windows::Win32::UI::WindowsAndMessaging::IDI_APPLICATION,
        )
    }?;

    // Create initial notification icon with tooltip
    let tray_icon = NotifyIcon::new()
        .window_handle(hwnd)
        .icon(icon)
        .tip("Initial Tooltip")
        .callback_message(WM_USER + 1);

    tray_icon.notify_add()?;

    // Modify the tooltip after a short delay (simulating an update)
    std::thread::sleep(std::time::Duration::from_secs(2));

    let updated_tray_icon = tray_icon.tip("Updated Tooltip - Modified!");
    updated_tray_icon.notify_modify()?;

    // Message loop
    let mut msg = unsafe { std::mem::zeroed() };
    // SAFETY: GetMessageW is safe to call with valid parameters
    while unsafe { GetMessageW(&mut msg, None, 0, 0) }.into() {
        // SAFETY: TranslateMessage and DispatchMessageW are safe to call with valid parameters
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    // Clean up
    updated_tray_icon.notify_delete()?;

    Ok(())
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // SAFETY: DefWindowProcW and PostQuitMessage are safe to call with valid parameters
    unsafe {
        match msg {
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
