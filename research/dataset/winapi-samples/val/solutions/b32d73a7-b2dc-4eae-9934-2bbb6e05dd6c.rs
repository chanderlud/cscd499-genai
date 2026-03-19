use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::WNDCLASSEXW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassW, ShowWindow, TranslateMessage, UnregisterClassW, CS_HREDRAW, CS_VREDRAW, MSG,
    SW_SHOW, WM_DESTROY, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

// Helper to create a static UTF-16 null-terminated string
fn static_wide_str(s: &str) -> &'static [u16] {
    let mut result = s.encode_utf16().collect::<Vec<_>>();
    result.push(0);
    Box::leak(result.into_boxed_slice())
}

// Window procedure for handling window messages
unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn main() -> Result<()> {
    unsafe {
        let hinstance = GetModuleHandleW(None)?;
        let class_name = static_wide_str("MinimalWindowClass");
        let window_title = static_wide_str("Minimal Window");

        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinstance.into(),
            lpszClassName: PCWSTR::from_raw(class_name.as_ptr()),
            style: CS_HREDRAW | CS_VREDRAW,
            ..Default::default()
        };

        let class_atom = RegisterClassW(&wnd_class as *const _ as *const _);
        if class_atom == 0 {
            return Err(windows::core::Error::from_thread());
        }

        // Create window
        let hwnd = CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
            PCWSTR::from_raw(class_name.as_ptr()),
            PCWSTR::from_raw(window_title.as_ptr()),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            100,
            100,
            400,
            300,
            None,
            None,
            Some(hinstance.into()),
            None,
        )?;

        // Show window
        if !ShowWindow(hwnd, SW_SHOW).as_bool() {
            return Err(windows::core::Error::from_thread());
        }

        // Message loop
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        UnregisterClassW(
            PCWSTR::from_raw(class_name.as_ptr()),
            Some(hinstance.into()),
        )?;
    }
    Ok(())
}
