use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{EnableWindow, IsWindowEnabled};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassExW, ShowWindow,
    TranslateMessage, BS_PUSHBUTTON, CW_USEDEFAULT, MSG, SW_SHOW, WINDOW_EX_STYLE, WM_DESTROY,
    WNDCLASSEXW, WS_CHILD, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;

        let class_name = wide_null(OsStr::new("ButtonEnabledExample"));
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        let atom = RegisterClassExW(&wc);
        if atom == 0 {
            return Err(Error::from_thread());
        }

        let window_title = wide_null(OsStr::new("Button Enabled State Example"));
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_title.as_ptr()),
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

        let button_text = wide_null(OsStr::new("Click Me"));
        let button_class = wide_null(OsStr::new("BUTTON"));
        let button_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(button_class.as_ptr()),
            PCWSTR(button_text.as_ptr()),
            windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(
                (WS_CHILD | WS_VISIBLE).0 | BS_PUSHBUTTON as u32,
            ),
            50,
            50,
            200,
            50,
            Some(hwnd),
            None,
            Some(instance.into()),
            None,
        )?;

        // Check initial enabled state
        let initially_enabled = IsWindowEnabled(button_hwnd);
        println!("Button initially enabled: {}", initially_enabled.as_bool());

        // Disable the button
        EnableWindow(button_hwnd, false);

        // Verify it's disabled
        let after_disable = IsWindowEnabled(button_hwnd);
        println!("Button after disabling: {}", after_disable.as_bool());

        // Re-enable the button
        EnableWindow(button_hwnd, true);

        // Verify it's enabled again
        let after_enable = IsWindowEnabled(button_hwnd);
        println!("Button after re-enabling: {}", after_enable.as_bool());

        ShowWindow(hwnd, SW_SHOW);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Ok(())
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => {
            std::process::exit(0);
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
