use windows::core::{Result, Error, PCWSTR};
use windows::Win32::Foundation::{HWND, HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassExW, TranslateMessage, CW_USEDEFAULT, MSG, WINDOW_EX_STYLE, WINDOW_STYLE,
    WM_DESTROY, WNDCLASSEXW,
};
use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

// Window procedure callback
unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn create_window() -> Result<HWND> {
    let class_name = wide_null(OsStr::new("SampleWindowClass"));
    let window_name = wide_null(OsStr::new("Sample Window"));

    let hmodule = unsafe { GetModuleHandleW(None) }?;
    let hinstance = HINSTANCE(hmodule.0);

    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: Default::default(),
        lpfnWndProc: Some(wndproc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance,
        hIcon: Default::default(),
        hCursor: Default::default(),
        hbrBackground: Default::default(),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        hIconSm: Default::default(),
    };

    let atom = unsafe { RegisterClassExW(&wc) };
    if atom == 0 {
        return Err(Error::from_thread());
    }

    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_name.as_ptr()),
            WINDOW_STYLE::default(),
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            Some(hinstance),
            None,
        )
    }?;

    Ok(hwnd)
}

fn run_message_loop() -> Result<()> {
    let mut msg = MSG::default();
    loop {
        let result = unsafe { GetMessageW(&mut msg, None, 0, 0) };
        if result.0 == 0 {
            // WM_QUIT received
            break;
        } else if result.0 == -1 {
            // Error occurred
            return Err(Error::from_thread());
        } else {
            unsafe {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
    Ok(())
}

// This function is called by the test framework
pub fn run() -> Result<()> {
    let hwnd = create_window()?;
    
    // Show the window (optional, but good for testing)
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_SHOWDEFAULT};
        ShowWindow(hwnd, SW_SHOWDEFAULT);
    }
    
    run_message_loop()?;
    Ok(())
}