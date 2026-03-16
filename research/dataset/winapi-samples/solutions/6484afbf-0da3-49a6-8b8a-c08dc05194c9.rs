// Get and set button visibility using IsWindowVisible and ShowWindow

use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::HBRUSH;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, IsWindowVisible, ShowWindow,
    TranslateMessage, CW_USEDEFAULT, MSG, SW_HIDE, SW_SHOW, WINDOW_EX_STYLE, WINDOW_STYLE,
    WM_DESTROY, WM_QUIT, WS_CHILD, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    // Register window class
    let class_name = wide_null(OsStr::new("VisibilityExampleClass"));
    let window_title = wide_null(OsStr::new("Button Visibility Example"));

    unsafe {
        // Create main window
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
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
            None,
        )?;

        // Create button
        let button_class = wide_null(OsStr::new("BUTTON"));
        let button_text = wide_null(OsStr::new("Toggle Visibility"));
        let button_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(button_class.as_ptr()),
            PCWSTR(button_text.as_ptr()),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(0x80000000), // BS_PUSHBUTTON
            50,
            50,
            200,
            50,
            Some(hwnd),
            None,
            None,
            None,
        )?;

        // Check initial visibility
        let is_visible = IsWindowVisible(button_hwnd);
        println!("Button initially visible: {}", is_visible.as_bool());

        // Hide the button
        ShowWindow(button_hwnd, SW_HIDE);
        let is_visible_after_hide = IsWindowVisible(button_hwnd);
        println!(
            "Button visible after SW_HIDE: {}",
            is_visible_after_hide.as_bool()
        );

        // Show the button again
        ShowWindow(button_hwnd, SW_SHOW);
        let is_visible_after_show = IsWindowVisible(button_hwnd);
        println!(
            "Button visible after SW_SHOW: {}",
            is_visible_after_show.as_bool()
        );

        // Message loop
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Ok(())
    }
}
