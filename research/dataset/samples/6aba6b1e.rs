use windows::core::w;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, PAINTSTRUCT};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{VIRTUAL_KEY, VK_ESCAPE, VK_RETURN};
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW,
    GetWindowLongPtrW, LoadCursorW, PostQuitMessage, RegisterClassW, SendMessageW,
    SetForegroundWindow, SetWindowLongPtrW, ShowWindow, TranslateMessage, CS_HREDRAW, CS_VREDRAW,
    CW_USEDEFAULT, GWLP_USERDATA, GWL_WNDPROC, IDC_ARROW, LBS_NOTIFY, LB_ADDSTRING, LB_ERR,
    LB_GETCURSEL, LB_GETITEMDATA, LB_SETCURSEL, LB_SETITEMDATA, MSG, SW_SHOW, WINDOW_EX_STYLE,
    WINDOW_STYLE, WM_CREATE, WM_DESTROY, WM_KEYUP, WM_PAINT, WNDCLASSW, WS_CHILD, WS_POPUP,
    WS_VISIBLE, WS_VSCROLL,
};

struct WindowInfo {
    hwnd: HWND,
    title: Vec<u16>,
}

struct AppContext {
    list_box: HWND,
    windows: Vec<WindowInfo>,
}

fn main() -> windows::core::Result<()> {
    unsafe {
        let instance: HINSTANCE = GetModuleHandleW(None)?.into();
        let window_class = w!("ForegroundExample");

        let wnd_class = WNDCLASSW {
            hInstance: instance,
            lpszClassName: window_class,
            lpfnWndProc: Some(window_proc),
            style: CS_HREDRAW | CS_VREDRAW,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            ..Default::default()
        };

        RegisterClassW(&wnd_class);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            window_class,
            w!("Foreground Example"),
            WS_POPUP | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            Some(instance),
            None,
        )?;

        ShowWindow(hwnd, SW_SHOW);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Ok(())
    }
}

extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            unsafe {
                let list_box = CreateWindowExW(
                    WINDOW_EX_STYLE(0),
                    w!("ListBox"),
                    w!(""),
                    WS_CHILD
                        | WS_VISIBLE
                        | WINDOW_STYLE(LBS_NOTIFY.try_into().unwrap())
                        | WS_VSCROLL,
                    10,
                    10,
                    380,
                    280,
                    Some(hwnd),
                    None,
                    Some(GetModuleHandleW(None).unwrap().into()),
                    None,
                )
                .unwrap();

                let windows = vec![
                    WindowInfo {
                        hwnd: HWND(0x1234 as *mut _), // Dummy handle for example
                        title: wide_null("Notepad"),
                    },
                    WindowInfo {
                        hwnd: HWND(0x5678 as *mut _), // Dummy handle for example
                        title: wide_null("Calculator"),
                    },
                ];

                for (i, window) in windows.iter().enumerate() {
                    SendMessageW(
                        list_box,
                        LB_ADDSTRING,
                        Some(WPARAM(0)),
                        Some(LPARAM(window.title.as_ptr() as isize)),
                    );
                    SendMessageW(
                        list_box,
                        LB_SETITEMDATA,
                        Some(WPARAM(i)),
                        Some(LPARAM(window.hwnd.0 as isize)),
                    );
                }

                SendMessageW(list_box, LB_SETCURSEL, Some(WPARAM(0)), Some(LPARAM(0)));

                let app_context = Box::new(AppContext { list_box, windows });
                let app_context_ptr = Box::into_raw(app_context);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, app_context_ptr as isize);
                SetWindowLongPtrW(list_box, GWLP_USERDATA, app_context_ptr as isize);

                SetWindowLongPtrW(list_box, GWL_WNDPROC, list_box_proc as isize);
            }
            LRESULT(0)
        }

        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            unsafe {
                let _hdc = BeginPaint(hwnd, &mut ps);
                _ = EndPaint(hwnd, &ps);
            }
            LRESULT(0)
        }

        WM_DESTROY => {
            unsafe {
                let app_context_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppContext;
                if !app_context_ptr.is_null() {
                    let _ = Box::from_raw(app_context_ptr);
                }
                PostQuitMessage(0);
            }
            LRESULT(0)
        }

        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

extern "system" fn list_box_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_KEYUP {
        let key = wparam.0 as u32;
        let vkey = VIRTUAL_KEY(key as u16);

        if vkey == VK_RETURN {
            unsafe {
                let selected_index =
                    SendMessageW(hwnd, LB_GETCURSEL, Some(WPARAM(0)), Some(LPARAM(0))).0 as i32;
                if selected_index != LB_ERR {
                    let window_handle = SendMessageW(
                        hwnd,
                        LB_GETITEMDATA,
                        Some(WPARAM(selected_index as usize)),
                        Some(LPARAM(0)),
                    );
                    let target_hwnd = HWND(window_handle.0 as *mut _);
                    // Bring the selected window to the foreground
                    let _ = SetForegroundWindow(target_hwnd);
                }
            }
            return LRESULT(0);
        } else if vkey == VK_ESCAPE {
            unsafe { PostQuitMessage(0) };
            return LRESULT(0);
        }
    }

    unsafe {
        let app_context_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const AppContext;
        if !app_context_ptr.is_null() {
            let original_proc = (*app_context_ptr).list_box;
            let proc_ptr = GetWindowLongPtrW(original_proc, GWL_WNDPROC);
            let proc = Some(std::mem::transmute(proc_ptr));
            CallWindowProcW(proc, hwnd, msg, wparam, lparam)
        } else {
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }
}

fn wide_null(s: &str) -> Vec<u16> {
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
    OsStr::new(s).encode_wide().chain(once(0)).collect()
}
