use windows::core::w;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, EndPaint, GetStockObject, COLOR_WINDOWFRAME, FONT_CHARSET,
    FONT_CLIP_PRECISION, FONT_OUTPUT_PRECISION, FONT_QUALITY, GET_STOCK_OBJECT_FLAGS, HBRUSH,
    PAINTSTRUCT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{VIRTUAL_KEY, VK_RETURN};
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW,
    GetWindowLongPtrW, LoadCursorW, PostQuitMessage, RegisterClassW, SendMessageW,
    SetWindowLongPtrW, SetWindowTextW, ShowWindow, TranslateMessage, CREATESTRUCTW, CS_HREDRAW,
    CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, GWL_WNDPROC, IDC_ARROW, LBS_NOTIFY, LB_ADDSTRING,
    LB_ERR, LB_GETCURSEL, LB_GETITEMDATA, LB_RESETCONTENT, LB_SETCURSEL, LB_SETITEMDATA, MSG,
    SW_SHOW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_CREATE, WM_DESTROY, WM_KEYUP, WM_PAINT, WM_SETFONT,
    WNDCLASSW, WS_CHILD, WS_POPUP, WS_VISIBLE, WS_VSCROLL,
};

const TEXT_BOX_HEIGHT: i32 = 36;

struct UIElement {
    hwnd: HWND,
    original_proc_ptr: isize,
}

struct AppContext {
    list_box: UIElement,
    items: Vec<String>,
}

struct WindowData {
    main_window: HWND,
}

static mut WINDOW_DATA: Option<WindowData> = None;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        if WINDOW_DATA.is_some() {
            panic!("Window already created!");
        }

        let instance: HINSTANCE = GetModuleHandleW(None)?.into();
        let window_class = w!("ListBoxExample");

        let background = GetStockObject(GET_STOCK_OBJECT_FLAGS(COLOR_WINDOWFRAME.0));
        if background.is_invalid() {
            return Err("Failed to get stock object".into());
        }

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
            w!("ListBox Data Example"),
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

        WINDOW_DATA = Some(WindowData { main_window: hwnd });

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
                let items = vec![
                    "Document 1".to_string(),
                    "Document 2".to_string(),
                    "Image File".to_string(),
                    "Spreadsheet".to_string(),
                    "Presentation".to_string(),
                ];

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
                    Some(GetModuleHandleW(None).unwrap().into()),
                    None,
                )
                .unwrap();

                let original_proc_ptr =
                    SetWindowLongPtrW(list_box, GWL_WNDPROC, list_box_proc as isize);

                let app_context = Box::new(AppContext {
                    list_box: UIElement {
                        hwnd: list_box,
                        original_proc_ptr,
                    },
                    items,
                });

                let app_context_ptr = Box::into_raw(app_context);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, app_context_ptr as isize);
                SetWindowLongPtrW(list_box, GWLP_USERDATA, app_context_ptr as isize);

                let font = CreateFontW(
                    -16,
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
                );

                SendMessageW(
                    list_box,
                    WM_SETFONT,
                    Some(WPARAM(font.0 as usize)),
                    Some(LPARAM(1)),
                );

                populate_list_box(list_box, &(*app_context_ptr).items);

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
                    drop(Box::from_raw(app_context_ptr));
                }
                PostQuitMessage(0);
                LRESULT(0)
            }

            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

unsafe fn populate_list_box(list_box: HWND, items: &[String]) {
    SendMessageW(list_box, LB_RESETCONTENT, Some(WPARAM(0)), Some(LPARAM(0)));

    for (i, item) in items.iter().enumerate() {
        let wide_item = wide_null(item);
        SendMessageW(
            list_box,
            LB_ADDSTRING,
            Some(WPARAM(0)),
            Some(LPARAM(wide_item.as_ptr() as isize)),
        );

        SendMessageW(
            list_box,
            LB_SETITEMDATA,
            Some(WPARAM(i)),
            Some(LPARAM(i as isize)),
        );
    }

    SendMessageW(list_box, LB_SETCURSEL, Some(WPARAM(0)), Some(LPARAM(0)));
}

extern "system" fn list_box_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        if msg == WM_KEYUP {
            let key = wparam.0 as u32;
            let vkey = VIRTUAL_KEY(key as u16);

            if vkey == VK_RETURN {
                let selected_index =
                    SendMessageW(hwnd, LB_GETCURSEL, Some(WPARAM(0)), Some(LPARAM(0))).0 as i32;

                if selected_index != LB_ERR {
                    let item_data = SendMessageW(
                        hwnd,
                        LB_GETITEMDATA,
                        Some(WPARAM(selected_index as usize)),
                        Some(LPARAM(0)),
                    );

                    let index = item_data.0 as usize;
                    let app_context_ptr =
                        GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const AppContext;

                    if !app_context_ptr.is_null() {
                        let app_context = &*app_context_ptr;
                        if index < app_context.items.len() {
                            let selected_text = &app_context.items[index];
                            let wide_text = wide_null(selected_text);
                            let main_window = (*WINDOW_DATA.as_ref().unwrap()).main_window;
                            SetWindowTextW(main_window, windows::core::PCWSTR(wide_text.as_ptr()));
                        }
                    }
                }
                return LRESULT(0);
            }
        }

        let app_context_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const AppContext;
        if !app_context_ptr.is_null() {
            let app_context = &*app_context_ptr;
            let proc = std::mem::transmute(app_context.list_box.original_proc_ptr);
            CallWindowProcW(proc, hwnd, msg, wparam, lparam)
        } else {
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }
}

fn wide_null(s: &str) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(once(0))
        .collect()
}
