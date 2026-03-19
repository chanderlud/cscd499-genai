use windows::core::w;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, GetStockObject, COLOR_WINDOWFRAME, GET_STOCK_OBJECT_FLAGS, HBRUSH,
    PAINTSTRUCT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VIRTUAL_KEY, VK_DOWN, VK_ESCAPE, VK_RETURN, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW,
    GetWindowLongPtrW, LoadCursorW, PostQuitMessage, RegisterClassW, SendMessageW,
    SetWindowLongPtrW, ShowWindow, TranslateMessage, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT,
    GWLP_USERDATA, GWL_WNDPROC, IDC_ARROW, LBS_NOTIFY, LB_ADDSTRING, LB_ERR, LB_GETCURSEL,
    LB_GETITEMDATA, LB_SETCURSEL, MSG, SW_SHOW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_CREATE,
    WM_DESTROY, WM_KEYDOWN, WM_KEYUP, WM_PAINT, WNDCLASSW, WS_CHILD, WS_POPUP, WS_VISIBLE,
    WS_VSCROLL,
};

struct UIElement {
    original_proc_ptr: isize,
}

struct AppContext {
    list_box: UIElement,
}

fn main() -> windows::core::Result<()> {
    unsafe {
        let instance: HINSTANCE = GetModuleHandleW(None)?.into();
        let window_class = w!("ListBoxSubclassExample");
        let background = GetStockObject(GET_STOCK_OBJECT_FLAGS(COLOR_WINDOWFRAME.0));

        let wnd_class = WNDCLASSW {
            hInstance: instance,
            lpszClassName: window_class,
            lpfnWndProc: Some(window_proc),
            style: CS_HREDRAW | CS_VREDRAW,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hbrBackground: HBRUSH(background.0),
            ..Default::default()
        };

        RegisterClassW(&wnd_class);

        let hwnd: HWND = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            window_class,
            w!("ListBox Subclass Example"),
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

        let _ = ShowWindow(hwnd, SW_SHOW);

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
                let mut rect = RECT::default();
                GetClientRect(hwnd, &mut rect).unwrap();

                let list_box = CreateWindowExW(
                    WINDOW_EX_STYLE(0),
                    w!("ListBox"),
                    w!(""),
                    WS_CHILD
                        | WS_VISIBLE
                        | WINDOW_STYLE(LBS_NOTIFY.try_into().unwrap())
                        | WS_VSCROLL,
                    0,
                    0,
                    rect.right,
                    rect.bottom,
                    Some(hwnd),
                    None,
                    Some(HINSTANCE(GetModuleHandleW(None).unwrap().0)),
                    None,
                )
                .unwrap();

                let original_proc_ptr =
                    SetWindowLongPtrW(list_box, GWL_WNDPROC, list_box_proc as *const () as isize);

                let app_context = Box::new(AppContext {
                    list_box: UIElement { original_proc_ptr },
                });

                let app_context_ptr = Box::into_raw(app_context);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, app_context_ptr as isize);
                SetWindowLongPtrW(list_box, GWLP_USERDATA, app_context_ptr as isize);

                for i in 0..10 {
                    let item = format!("Item {}", i + 1);
                    let wide_item: Vec<u16> =
                        item.encode_utf16().chain(std::iter::once(0)).collect();
                    SendMessageW(
                        list_box,
                        LB_ADDSTRING,
                        Some(WPARAM(0)),
                        Some(LPARAM(wide_item.as_ptr() as isize)),
                    );
                }

                SendMessageW(list_box, LB_SETCURSEL, Some(WPARAM(0)), Some(LPARAM(0)));
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
    unsafe {
        let app_context_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const AppContext;
        if app_context_ptr.is_null() {
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }

        let app_context = &*app_context_ptr;

        if msg == WM_KEYUP {
            let key = wparam.0 as u32;
            if key == VK_ESCAPE.0 as u32 {
                PostQuitMessage(0);
                return LRESULT(0);
            }
        }

        if msg == WM_KEYDOWN {
            let key = wparam.0 as u32;
            let vkey = VIRTUAL_KEY(key as u16);

            if vkey == VK_UP || vkey == VK_DOWN {
                let proc = std::mem::transmute::<
                    isize,
                    Option<unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT>,
                >(app_context.list_box.original_proc_ptr);
                return CallWindowProcW(proc, hwnd, msg, wparam, lparam);
            }
        }

        if msg == WM_KEYUP {
            let key = wparam.0 as u32;
            let vkey = VIRTUAL_KEY(key as u16);

            if vkey == VK_RETURN {
                let selected_index =
                    SendMessageW(hwnd, LB_GETCURSEL, Some(WPARAM(0)), Some(LPARAM(0))).0 as i32;
                if selected_index != LB_ERR {
                    let ptr = SendMessageW(
                        hwnd,
                        LB_GETITEMDATA,
                        Some(WPARAM(selected_index as usize)),
                        Some(LPARAM(0)),
                    );
                    let _ = ptr;
                }
                return LRESULT(0);
            }
        }

        let proc = std::mem::transmute::<
            isize,
            Option<unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT>,
        >(app_context.list_box.original_proc_ptr);
        CallWindowProcW(proc, hwnd, msg, wparam, lparam)
    }
}
