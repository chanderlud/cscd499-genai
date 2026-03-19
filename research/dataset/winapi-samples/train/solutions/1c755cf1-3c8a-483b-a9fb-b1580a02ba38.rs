use windows::core::{w, Result};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, PAINTSTRUCT};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, GetParent,
    GetWindowLongPtrW, GetWindowTextLengthW, GetWindowTextW, PostQuitMessage, RegisterClassW,
    SetWindowLongPtrW, SetWindowTextW, ShowWindow, TranslateMessage, CS_HREDRAW, CS_VREDRAW,
    CW_USEDEFAULT, ES_AUTOHSCROLL, GWLP_USERDATA, GWL_WNDPROC, MSG, SW_SHOW, WINDOW_EX_STYLE,
    WINDOW_STYLE, WM_CREATE, WM_DESTROY, WM_KEYUP, WM_PAINT, WNDCLASSW, WS_CHILD,
    WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

fn get_str_from_buffer(buffer_with_null_ter: &[u16]) -> String {
    String::from_utf16_lossy(&buffer_with_null_ter[0..buffer_with_null_ter.len() - 1])
}

struct AppContext {
    original_edit_proc: isize,
}

extern "system" fn main_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => {
                let instance = GetModuleHandleW(None).unwrap();

                // Create edit control
                let edit_control = CreateWindowExW(
                    WINDOW_EX_STYLE(0),
                    w!("Edit"),
                    w!("Type here..."),
                    WS_CHILD | WS_VISIBLE | WINDOW_STYLE(ES_AUTOHSCROLL as u32),
                    10,
                    10,
                    300,
                    25,
                    Some(hwnd),
                    None,
                    Some(HINSTANCE(instance.0)),
                    None,
                )
                .unwrap();

                // Store original edit control procedure
                let original_edit_proc = SetWindowLongPtrW(
                    edit_control,
                    GWL_WNDPROC,
                    edit_control_proc as *const () as isize,
                );

                // Create app context
                let app_context = Box::new(AppContext { original_edit_proc });

                // Store app context in main window
                let app_context_ptr = Box::into_raw(app_context);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, app_context_ptr as isize);

                LRESULT(0)
            }

            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let _hdc = BeginPaint(hwnd, &mut ps);
                _ = EndPaint(hwnd, &ps);
                LRESULT(0)
            }

            WM_DESTROY => {
                let app_context_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppContext;
                if !app_context_ptr.is_null() {
                    let _ = Box::from_raw(app_context_ptr);
                }
                PostQuitMessage(0);
                LRESULT(0)
            }

            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

extern "system" fn edit_control_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        // Get parent window and app context
        let parent = GetParent(hwnd).unwrap();
        let app_context_ptr = GetWindowLongPtrW(parent, GWLP_USERDATA) as *const AppContext;

        if !app_context_ptr.is_null() {
            let app_context = &*app_context_ptr;

            if msg == WM_KEYUP {
                // Get text from edit control
                let length = GetWindowTextLengthW(hwnd) as usize;
                if length > 0 {
                    let mut buffer = vec![0u16; length + 1];
                    GetWindowTextW(hwnd, &mut buffer);
                    let text = get_str_from_buffer(&buffer);

                    // Set parent window title to edit control text
                    let wide_text = wide_null(std::ffi::OsStr::new(&text));
                    let _ = SetWindowTextW(parent, windows::core::PCWSTR(wide_text.as_ptr()));
                } else {
                    // Clear title if edit control is empty
                    let _ = SetWindowTextW(parent, w!(""));
                }
            }

            // Call original edit control procedure
            let original_proc = Some(std::mem::transmute::<
                isize,
                unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT,
            >(app_context.original_edit_proc));
            CallWindowProcW(original_proc, hwnd, msg, wparam, lparam)
        } else {
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;

        let window_class = w!("EditTitleExample");

        let wnd_class = WNDCLASSW {
            hInstance: instance.into(),
            lpszClassName: window_class,
            lpfnWndProc: Some(main_window_proc),
            style: CS_HREDRAW | CS_VREDRAW,
            ..Default::default()
        };

        RegisterClassW(&wnd_class);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            window_class,
            w!("Edit Control Title Example"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            200,
            None,
            None,
            Some(HINSTANCE(instance.0)),
            None,
        )?;

        let _ = ShowWindow(hwnd, SW_SHOW);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Ok(())
    }
}
