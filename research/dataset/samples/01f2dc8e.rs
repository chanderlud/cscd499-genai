// TITLE: Setting Focus to Child Controls and Routing Keyboard Input

use windows::core::w;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, EndPaint, GetStockObject, COLOR_WINDOWFRAME, FONT_CHARSET,
    FONT_CLIP_PRECISION, FONT_OUTPUT_PRECISION, FONT_QUALITY, GET_STOCK_OBJECT_FLAGS, HBRUSH,
    HFONT, PAINTSTRUCT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{SetFocus, VIRTUAL_KEY, VK_DOWN, VK_UP};
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW,
    GetWindowLongPtrW, LoadCursorW, PostQuitMessage, RegisterClassW, SendMessageW,
    SetWindowLongPtrW, ShowWindow, TranslateMessage, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW,
    CW_USEDEFAULT, GWLP_USERDATA, GWL_WNDPROC, IDC_ARROW, LBS_NOTIFY, LB_SETCURSEL, MSG, SW_SHOW,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_CREATE, WM_DESTROY, WM_KEYDOWN, WM_KEYUP, WM_PAINT,
    WM_SETFONT, WNDCLASSW, WS_CHILD, WS_POPUP, WS_VISIBLE, WS_VSCROLL,
};

const TEXT_BOX_HEIGHT: i32 = 36;

struct UIElement {
    hwnd: HWND,
    original_proc_ptr: isize,
}

struct AppContext {
    text_box: UIElement,
    list_box: UIElement,
}

fn main() -> windows::core::Result<()> {
    unsafe {
        let instance: HINSTANCE = GetModuleHandleW(None)?.into();
        let window_class = w!("FocusExample");
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

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            window_class,
            w!("Focus Routing Example"),
            WS_POPUP | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            600,
            400,
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
    unsafe {
        match msg {
            WM_CREATE => {
                let cs = lparam.0 as *const CREATESTRUCTW;
                let mut rect = RECT::default();
                GetClientRect(hwnd, &mut rect).unwrap();

                let text_box = create_text_box(hwnd, rect);
                let list_box = create_list_box(hwnd, rect);

                let app_context = Box::new(AppContext { text_box, list_box });
                let app_context_ptr = Box::into_raw(app_context) as isize;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, app_context_ptr);

                let font = create_font();
                SendMessageW(
                    hwnd,
                    WM_SETFONT,
                    Some(WPARAM(font.0 as usize)),
                    Some(LPARAM(1)),
                );

                SetFocus(Some(hwnd));
                LRESULT(0)
            }
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let _hdc = BeginPaint(hwnd, &mut ps);
                _ = EndPaint(hwnd, &ps);
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

unsafe fn create_text_box(main_window: HWND, rect: RECT) -> UIElement {
    let text_box = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("Edit"),
        w!(""),
        WS_CHILD | WS_VISIBLE,
        0,
        0,
        rect.right,
        TEXT_BOX_HEIGHT,
        Some(main_window),
        None,
        Some(GetModuleHandleW(None).unwrap().into()),
        None,
    )
    .unwrap();

    let original_proc_ptr = SetWindowLongPtrW(text_box, GWL_WNDPROC, text_box_proc as isize);
    UIElement {
        hwnd: text_box,
        original_proc_ptr,
    }
}

unsafe fn create_list_box(main_window: HWND, rect: RECT) -> UIElement {
    let list_box = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("ListBox"),
        w!(""),
        WS_CHILD | WS_VISIBLE | WINDOW_STYLE(LBS_NOTIFY.try_into().unwrap()) | WS_VSCROLL,
        0,
        TEXT_BOX_HEIGHT,
        rect.right,
        rect.bottom - TEXT_BOX_HEIGHT,
        Some(main_window),
        None,
        Some(GetModuleHandleW(None).unwrap().into()),
        None,
    )
    .unwrap();

    let original_proc_ptr = SetWindowLongPtrW(list_box, GWL_WNDPROC, list_box_proc as isize);
    UIElement {
        hwnd: list_box,
        original_proc_ptr,
    }
}

unsafe fn create_font() -> HFONT {
    CreateFontW(
        -24,
        0,
        0,
        0,
        400,
        0,
        0,
        0,
        FONT_CHARSET(0),
        FONT_OUTPUT_PRECISION(0),
        FONT_CLIP_PRECISION(0),
        FONT_QUALITY(0),
        0,
        w!("Segoe UI"),
    )
}

extern "system" fn text_box_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        let app_context_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
        if app_context_ptr != 0 {
            let app_context = &*(app_context_ptr as *const AppContext);
            if let Some(result) =
                route_nav_keys_to_list_box(app_context.list_box.hwnd, msg, wparam, lparam)
            {
                return result;
            }
            let proc = std::mem::transmute(app_context.text_box.original_proc_ptr);
            CallWindowProcW(proc, hwnd, msg, wparam, lparam)
        } else {
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }
}

unsafe extern "system" fn list_box_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let app_context_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
    if app_context_ptr != 0 {
        let app_context = &*(app_context_ptr as *const AppContext);
        if let Some(result) =
            route_key_up_to_text_box(app_context.text_box.hwnd, msg, wparam, lparam)
        {
            return result;
        }
        let proc = std::mem::transmute(app_context.list_box.original_proc_ptr);
        CallWindowProcW(proc, hwnd, msg, wparam, lparam)
    } else {
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

fn route_nav_keys_to_list_box(
    list_box: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> Option<LRESULT> {
    if msg == WM_KEYDOWN {
        let key = wparam.0 as u32;
        let vkey = VIRTUAL_KEY(key as u16);
        if vkey == VK_DOWN {
            unsafe {
                SetFocus(Some(list_box)).unwrap();
                SendMessageW(list_box, LB_SETCURSEL, Some(WPARAM(0)), Some(LPARAM(0)));
            }
            return Some(LRESULT(0));
        }
    }
    None
}

fn route_key_up_to_text_box(
    text_box: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> Option<LRESULT> {
    if msg == WM_KEYUP {
        let key = wparam.0 as u32;
        let vkey = VIRTUAL_KEY(key as u16);
        if vkey == VK_UP {
            unsafe {
                SetFocus(Some(text_box)).unwrap();
            }
            return Some(LRESULT(0));
        }
    }
    None
}
