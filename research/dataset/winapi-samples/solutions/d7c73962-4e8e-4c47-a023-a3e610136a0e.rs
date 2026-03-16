use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{ScreenToClient, UpdateWindow};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, GetWindowRect, PostQuitMessage,
    RegisterClassExW, SetWindowPos, ShowWindow, TranslateMessage, BS_PUSHBUTTON, HWND_TOP, MSG,
    SWP_NOZORDER, SW_SHOW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY, WNDCLASSEXW, WS_CHILD,
    WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

fn main() -> Result<()> {
    // Register window class
    let class_name = wide_null("SampleWindowClass");
    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        lpfnWndProc: Some(wndproc),
        hInstance: unsafe { windows::Win32::System::LibraryLoader::GetModuleHandleW(None)?.into() },
        lpszClassName: PCWSTR(class_name.as_ptr()),
        ..Default::default()
    };
    unsafe { RegisterClassExW(&wc) };

    // Create main window
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(wide_null("Button Size/Position Example").as_ptr()),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            100,
            100,
            400,
            300,
            None,
            None,
            Some(windows::Win32::System::LibraryLoader::GetModuleHandleW(None)?.into()),
            None,
        )?
    };

    // Create button
    let button_class = wide_null("BUTTON");
    let button_text = wide_null("Test Button");
    let button_hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(button_class.as_ptr()),
            PCWSTR(button_text.as_ptr()),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_PUSHBUTTON as u32),
            50,
            50,
            120,
            30,
            Some(hwnd),
            None,
            Some(windows::Win32::System::LibraryLoader::GetModuleHandleW(None)?.into()),
            None,
        )?
    };

    // Get initial button size and position
    let mut rect = RECT::default();
    unsafe { GetWindowRect(button_hwnd, &mut rect)? };

    // Convert screen coordinates to client coordinates relative to parent
    let mut top_left = POINT {
        x: rect.left,
        y: rect.top,
    };
    let mut bottom_right = POINT {
        x: rect.right,
        y: rect.bottom,
    };
    unsafe { ScreenToClient(hwnd, &mut top_left).ok()? };
    unsafe { ScreenToClient(hwnd, &mut bottom_right).ok()? };

    let width = bottom_right.x - top_left.x;
    let height = bottom_right.y - top_left.y;
    println!(
        "Initial button position: ({}, {}), size: ({}, {})",
        top_left.x, top_left.y, width, height
    );

    // Set new button size and position
    let new_x = 100;
    let new_y = 80;
    let new_width = 150;
    let new_height = 40;

    unsafe {
        SetWindowPos(
            button_hwnd,
            Some(HWND_TOP),
            new_x,
            new_y,
            new_width,
            new_height,
            SWP_NOZORDER,
        )?
    };

    println!(
        "Set button to position: ({}, {}), size: ({}, {})",
        new_x, new_y, new_width, new_height
    );

    // Verify the change
    unsafe { GetWindowRect(button_hwnd, &mut rect)? };
    let mut top_left = POINT {
        x: rect.left,
        y: rect.top,
    };
    let mut bottom_right = POINT {
        x: rect.right,
        y: rect.bottom,
    };
    unsafe { ScreenToClient(hwnd, &mut top_left).ok()? };
    unsafe { ScreenToClient(hwnd, &mut bottom_right).ok()? };

    let width = bottom_right.x - top_left.x;
    let height = bottom_right.y - top_left.y;
    println!(
        "New button position: ({}, {}), size: ({}, {})",
        top_left.x, top_left.y, width, height
    );

    unsafe { ShowWindow(hwnd, SW_SHOW) };
    unsafe { UpdateWindow(hwnd).ok()? };

    // Message loop
    let mut msg = MSG::default();
    while unsafe { GetMessageW(&mut msg, None, 0, 0) }.into() {
        unsafe { TranslateMessage(&msg) };
        unsafe { DispatchMessageW(&msg) };
    }

    Ok(())
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
