use windows::{
    core::{Result, PCWSTR},
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Shell::{
                Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
                NOTIFYICONDATAW,
            },
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, LoadIconW,
                PostQuitMessage, RegisterClassW, TranslateMessage, CW_USEDEFAULT, IDI_APPLICATION,
                MSG, WM_DESTROY, WM_USER, WNDCLASSW,
            },
        },
    },
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;

        let class_name = wide_null("TrayIconExample".as_ref());
        let wnd_class = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        RegisterClassW(&wnd_class);

        let hwnd = CreateWindowExW(
            Default::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(wide_null("Tray Icon Example".as_ref()).as_ptr()),
            Default::default(),
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        let icon1 = LoadIconW(None, IDI_APPLICATION)?;
        let icon2 = LoadIconW(None, IDI_APPLICATION)?;

        let mut data = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: 1,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
            uCallbackMessage: WM_USER + 1,
            hIcon: icon1,
            ..Default::default()
        };

        let tip = wide_null("Initial Icon".as_ref());
        data.szTip[..tip.len()].copy_from_slice(&tip);

        Shell_NotifyIconW(NIM_ADD, &data).ok()?;

        std::thread::sleep(std::time::Duration::from_secs(2));

        data.hIcon = icon2;
        let new_tip = wide_null("Modified Icon".as_ref());
        data.szTip[..new_tip.len()].copy_from_slice(&new_tip);

        Shell_NotifyIconW(NIM_MODIFY, &data).ok()?;

        std::thread::sleep(std::time::Duration::from_secs(2));

        Shell_NotifyIconW(NIM_DELETE, &data).ok()?;

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Ok(())
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
