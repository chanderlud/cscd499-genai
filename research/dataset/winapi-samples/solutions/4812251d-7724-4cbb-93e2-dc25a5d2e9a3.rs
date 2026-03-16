use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::Result;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::{EM_GETMODIFY, EM_SETLIMITTEXT, EM_SETMODIFY};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassExW, SendMessageW, ShowWindow, TranslateMessage, CW_USEDEFAULT, ES_AUTOHSCROLL,
    MSG, SW_SHOW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_CREATE, WM_DESTROY, WNDCLASSEXW, WS_BORDER,
    WS_CHILD, WS_OVERLAPPEDWINDOW, WS_TABSTOP, WS_VISIBLE,
};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;

        let class_name = wide_null(OsStr::new("ModifiedFlagExample"));
        let window_title = wide_null(OsStr::new("Edit Control Modified Flag Example"));

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            lpszClassName: windows::core::PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        RegisterClassExW(&wc);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            windows::core::PCWSTR(class_name.as_ptr()),
            windows::core::PCWSTR(window_title.as_ptr()),
            WS_VISIBLE | WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            100,
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        ShowWindow(hwnd, SW_SHOW);

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).into() {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }

        Ok(())
    }
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => {
                // Get the instance handle from the main window
                let instance = GetModuleHandleW(None).unwrap();

                // Create an edit control as a child window
                let edit_class = wide_null(OsStr::new("EDIT"));
                let edit_hwnd = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    windows::core::PCWSTR(edit_class.as_ptr()),
                    windows::core::PCWSTR::null(),
                    WS_VISIBLE
                        | WS_CHILD
                        | WS_BORDER
                        | WS_TABSTOP
                        | WINDOW_STYLE(ES_AUTOHSCROLL as u32),
                    10,
                    10,
                    380,
                    30,
                    Some(hwnd),
                    None,
                    Some(instance.into()),
                    None,
                )
                .unwrap();

                // Set a text limit for demonstration
                SendMessageW(
                    edit_hwnd,
                    EM_SETLIMITTEXT,
                    Some(WPARAM(100)),
                    Some(LPARAM(0)),
                );

                // Check initial modified state (should be false)
                let initial_modified =
                    SendMessageW(edit_hwnd, EM_GETMODIFY, Some(WPARAM(0)), Some(LPARAM(0)));
                println!("Initial modified state: {}", initial_modified.0 != 0);

                // Set the modified flag to true
                SendMessageW(edit_hwnd, EM_SETMODIFY, Some(WPARAM(1)), Some(LPARAM(0)));

                // Check modified state again (should be true)
                let after_set_modified =
                    SendMessageW(edit_hwnd, EM_GETMODIFY, Some(WPARAM(0)), Some(LPARAM(0)));
                println!("After setting modified flag: {}", after_set_modified.0 != 0);

                // Reset the modified flag to false
                SendMessageW(edit_hwnd, EM_SETMODIFY, Some(WPARAM(0)), Some(LPARAM(0)));

                // Verify it's false again
                let final_modified =
                    SendMessageW(edit_hwnd, EM_GETMODIFY, Some(WPARAM(0)), Some(LPARAM(0)));
                println!("After resetting modified flag: {}", final_modified.0 != 0);

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
