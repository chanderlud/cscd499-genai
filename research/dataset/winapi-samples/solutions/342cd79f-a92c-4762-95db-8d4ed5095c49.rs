// TITLE: Create a static label control with text and font

use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{GetStockObject, DEFAULT_GUI_FONT};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassExW, SendMessageW, SetWindowLongPtrW, ShowWindow, TranslateMessage, CREATESTRUCTW,
    CW_USEDEFAULT, GWLP_USERDATA, MSG, SW_SHOW, WM_CREATE, WM_DESTROY, WM_SETFONT, WNDCLASSEXW,
    WS_CHILD, WS_EX_LEFT, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

struct WindowData {
    label: HWND,
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            let cs = lparam.0 as *const CREATESTRUCTW;
            let data = (*cs).lpCreateParams as *mut WindowData;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, data as isize);

            // Create static label control
            let class_name = wide_null("STATIC");
            let text = wide_null("Hello, Windows!");
            let label = CreateWindowExW(
                WS_EX_LEFT,
                PCWSTR(class_name.as_ptr()),
                PCWSTR(text.as_ptr()),
                WS_VISIBLE | WS_CHILD,
                10,
                10,
                200,
                25,
                Some(hwnd),
                None,
                None,
                None,
            )
            .expect("Failed to create label");

            // Set label font
            let font = GetStockObject(DEFAULT_GUI_FONT);
            // SAFETY: font is a valid HGDIOBJ from GetStockObject
            let _ = SendMessageW(
                label,
                WM_SETFONT,
                Some(WPARAM(font.0 as usize)),
                Some(LPARAM(1)),
            );

            (*data).label = label;
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn main() -> Result<()> {
    // Register window class
    let class_name = wide_null("LabelExampleClass");
    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        lpfnWndProc: Some(wndproc),
        hInstance: unsafe { windows::Win32::System::LibraryLoader::GetModuleHandleW(None)? }.into(),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        ..Default::default()
    };

    // SAFETY: RegisterClassExW is a valid Win32 API call
    if unsafe { RegisterClassExW(&wc) } == 0 {
        return Err(Error::from_thread());
    }

    // Create window data
    let mut data = WindowData {
        label: HWND::default(),
    };

    // Create main window
    let window_title = wide_null("Label Example");
    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_LEFT,
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_title.as_ptr()),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            None,
            Some(&mut data as *mut _ as *mut _),
        )?
    };

    unsafe { ShowWindow(hwnd, SW_SHOW) };

    // Message loop
    let mut msg = MSG::default();
    // SAFETY: GetMessageW is a valid Win32 API call
    while unsafe { GetMessageW(&mut msg, None, 0, 0) }.into() {
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    Ok(())
}
