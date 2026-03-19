use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, PAINTSTRUCT};
use windows::Win32::Graphics::Gdi::{GetStockObject, HBRUSH, WHITE_BRUSH};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassExW, ShowWindow, TranslateMessage, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, MSG,
    SW_SHOW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY, WM_PAINT, WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
};

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

// Window procedure callback
unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => {
            // unsafe: PostQuitMessage is safe to call
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        WM_PAINT => {
            // unsafe: BeginPaint/EndPaint require valid HWND and PAINTSTRUCT
            unsafe {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);
                // Just validate the window area
                let _ = EndPaint(hwnd, &ps);
            }
            LRESULT(0)
        }
        _ => {
            // unsafe: DefWindowProcW requires valid parameters
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
    }
}

fn main() -> Result<()> {
    // unsafe: GetModuleHandleW with null gets current executable's module handle
    let hinstance = unsafe { GetModuleHandleW(None) }?;

    let class_name = wide_null("SampleWindowClass");

    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wndproc),
        hInstance: hinstance.into(),
        hCursor: Default::default(),
        hbrBackground: HBRUSH(unsafe { GetStockObject(WHITE_BRUSH) }.0),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        ..Default::default()
    };

    // unsafe: RegisterClassExW takes a valid WNDCLASSEXW pointer
    let atom = unsafe { RegisterClassExW(&wc) };
    if atom == 0 {
        return Err(Error::from_thread());
    }

    let window_title = wide_null("Sample Window");

    // unsafe: CreateWindowExW with valid parameters
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_title.as_ptr()),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            800,
            600,
            None,
            None,
            Some(hinstance.into()),
            None,
        )
    }?;

    // unsafe: ShowWindow with valid HWND
    unsafe { ShowWindow(hwnd, SW_SHOW) };

    // Message loop
    let mut msg = MSG::default();
    loop {
        // unsafe: GetMessageW with valid MSG pointer
        let result = unsafe { GetMessageW(&mut msg, None, 0, 0) };
        if result.0 == 0 {
            break;
        }
        if result.0 == -1 {
            return Err(Error::from_thread());
        }

        // unsafe: TranslateMessage and DispatchMessageW with valid MSG
        unsafe {
            let _ = TranslateMessage(&msg);
            let _ = DispatchMessageW(&msg);
        }
    }

    Ok(())
}
