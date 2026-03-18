use windows::{
    core::{Error, Result, PCWSTR},
    Win32::{
        Foundation::{FALSE, HWND, LPARAM, LRESULT, WPARAM},
        UI::{
            Shell::{
                Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE,
                NIM_SETFOCUS, NOTIFYICONDATAW,
            },
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, LoadIconW,
                PostQuitMessage, RegisterClassW, TranslateMessage, CW_USEDEFAULT, IDI_APPLICATION,
                MSG, WM_DESTROY, WM_USER, WNDCLASSW, WS_OVERLAPPEDWINDOW,
            },
        },
    },
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        // Register window class
        let class_name = wide_null("TrayIconFocusExample".as_ref());
        let wnd_class = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: Default::default(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };
        RegisterClassW(&wnd_class);

        // Create window
        let hwnd = CreateWindowExW(
            Default::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(wide_null("Tray Icon Focus Example".as_ref()).as_ptr()),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            None,
            None,
        )?;

        // Load system icon
        let icon = LoadIconW(None, IDI_APPLICATION)?;

        // Create notification icon data
        let mut nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: 1,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
            uCallbackMessage: WM_USER + 1,
            hIcon: icon,
            ..Default::default()
        };

        // Set tooltip
        let tip = wide_null("Click to focus".as_ref());
        let max_len = nid.szTip.len() - 1;
        if tip.len() <= max_len + 1 {
            nid.szTip[..tip.len()].copy_from_slice(&tip);
        } else {
            nid.szTip[..max_len].copy_from_slice(&tip[..max_len]);
            nid.szTip[max_len] = 0;
        }

        // Add notification icon
        if Shell_NotifyIconW(NIM_ADD, &nid) == FALSE {
            return Err(Error::from_thread());
        }

        // Set focus to the notification icon
        if Shell_NotifyIconW(NIM_SETFOCUS, &nid) == FALSE {
            // Clean up before returning error
            Shell_NotifyIconW(NIM_DELETE, &nid);
            return Err(Error::from_thread());
        }

        // Message loop
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Clean up notification icon
        Shell_NotifyIconW(NIM_DELETE, &nid);

        Ok(())
    }
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            val if val == WM_USER + 1 => {
                // Notification icon callback message
                println!("Notification icon interaction: {}", lparam.0);
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
