use windows::core::w;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, EndPaint, GetStockObject, COLOR_WINDOWFRAME, GET_STOCK_OBJECT_FLAGS,
    HBRUSH, HFONT, PAINTSTRUCT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SetFocus, VIRTUAL_KEY, VK_DOWN, VK_RETURN, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW,
    GetWindowLongPtrW, GetWindowTextLengthW, GetWindowTextW, LoadCursorW, PostQuitMessage,
    RegisterClassW, SendMessageW, SetWindowLongPtrW, ShowWindow, TranslateMessage, CREATESTRUCTW,
    CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, GWL_WNDPROC, IDC_ARROW, LBS_NOTIFY,
    LB_ADDSTRING, LB_ERR, LB_GETCURSEL, LB_GETITEMDATA, LB_RESETCONTENT, LB_SETCURSEL,
    LB_SETITEMDATA, MSG, SW_SHOW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_CHAR, WM_CREATE, WM_DESTROY,
    WM_KEYDOWN, WM_KEYUP, WM_PAINT, WM_SETFONT, WNDCLASSW, WS_CHILD, WS_POPUP, WS_VISIBLE,
    WS_VSCROLL,
};

const VK_ESCAPE: u32 = 0x1B;
const TEXT_BOX_HEIGHT: i32 = 36;

struct UIElement {
    hwnd: HWND,
    original_proc_ptr: isize,
}

struct AppContext {
    text_box: UIElement,
    list_box: UIElement,
    open_windows: Vec<String>,
}

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> windows::core::Result<()> {
    unsafe {
        let instance: HINSTANCE = GetModuleHandleW(None)?.into();
        let window_class = w!("FuzzySearchWindow");

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

        let open_windows = vec![
            "Notepad".to_string(),
            "Calculator".to_string(),
            "Command Prompt".to_string(),
            "File Explorer".to_string(),
            "Task Manager".to_string(),
        ];

        let open_windows_ptr = Box::into_raw(Box::new(open_windows));

        let hwnd: HWND = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            window_class,
            w!("Fuzzy Search Example"),
            WS_POPUP | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            600,
            400,
            None,
            None,
            Some(instance),
            Some(open_windows_ptr as _),
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
    if let Some(result) = default_window_proc(hwnd, msg, wparam, lparam) {
        return result;
    }

    unsafe {
        let app_context = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const AppContext;

        if !app_context.is_null() {
            let app_context = &(*app_context);

            if let Some(result) =
                route_nav_keys_to_list_box(app_context.list_box.hwnd, msg, wparam, lparam)
            {
                return result;
            }

            if let Some(result) =
                route_key_up_to_text_box(app_context.text_box.hwnd, msg, wparam, lparam)
            {
                return result;
            }

            if let Some(result) = default_select_on_return(app_context.list_box.hwnd, msg, wparam) {
                return result;
            }
        }
    }

    match msg {
        WM_CREATE => {
            unsafe {
                let cs = lparam.0 as *const CREATESTRUCTW;
                let open_windows = Box::from_raw((*cs).lpCreateParams as *mut Vec<String>);

                let mut rect = RECT::default();
                GetClientRect(hwnd, &mut rect).unwrap();

                let text_box = create_text_box(hwnd, rect);
                let list_box = create_list_box(hwnd, rect);

                let app_context = Box::new(AppContext {
                    list_box,
                    text_box,
                    open_windows: *open_windows,
                });

                let app_context_ptr = &*app_context as *const AppContext as _;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, app_context_ptr);
                SetWindowLongPtrW(app_context.list_box.hwnd, GWLP_USERDATA, app_context_ptr);
                SetWindowLongPtrW(app_context.text_box.hwnd, GWLP_USERDATA, app_context_ptr);

                let font = create_font();
                SendMessageW(
                    app_context.text_box.hwnd,
                    WM_SETFONT,
                    Some(WPARAM(font.0 as usize)),
                    Some(LPARAM(1)),
                );
                SendMessageW(
                    app_context.list_box.hwnd,
                    WM_SETFONT,
                    Some(WPARAM(font.0 as usize)),
                    Some(LPARAM(1)),
                );

                SetFocus(Some(app_context.text_box.hwnd)).unwrap();

                updated_list_box(app_context.list_box.hwnd, app_context.open_windows.iter());

                Box::leak(app_context);
            };

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

        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
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

    let original_proc_ptr =
        SetWindowLongPtrW(text_box, GWL_WNDPROC, text_box_proc as *const () as isize);

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

    let original_proc_ptr =
        SetWindowLongPtrW(list_box, GWL_WNDPROC, list_box_proc as *const () as isize);

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
        windows::Win32::Graphics::Gdi::FONT_CHARSET(0),
        windows::Win32::Graphics::Gdi::FONT_OUTPUT_PRECISION(0),
        windows::Win32::Graphics::Gdi::FONT_CLIP_PRECISION(0),
        windows::Win32::Graphics::Gdi::FONT_QUALITY(0),
        0,
        w!("Segoe UI"),
    )
}

fn default_exit_on_esc(msg: u32, wparam: WPARAM) -> Option<LRESULT> {
    if msg == WM_KEYUP {
        let key = wparam.0 as u32;
        if key == VK_ESCAPE {
            unsafe { PostQuitMessage(0) };
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
    if msg == WM_KEYUP || msg == WM_CHAR {
        let key = wparam.0 as u32;
        let vkey = VIRTUAL_KEY(key as u16);

        if vkey == VK_RETURN {
            return None;
        }

        if vkey == VK_UP || vkey == VK_DOWN {
            return None;
        }

        unsafe {
            SetFocus(Some(text_box)).unwrap();
            return Some(SendMessageW(text_box, msg, Some(wparam), Some(lparam)));
        }
    }
    None
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

        if vkey == VK_UP || vkey == VK_DOWN {
            unsafe {
                SetFocus(Some(list_box)).unwrap();
                return Some(SendMessageW(list_box, msg, Some(wparam), Some(lparam)));
            }
        }
    }
    None
}

fn default_select_on_return(list_box: HWND, msg: u32, wparam: WPARAM) -> Option<LRESULT> {
    if msg == WM_KEYUP {
        let key = wparam.0 as u32;
        let vkey = VIRTUAL_KEY(key as u16);

        if vkey == VK_RETURN {
            let selected_index = unsafe {
                SendMessageW(list_box, LB_GETCURSEL, Some(WPARAM(0)), Some(LPARAM(0))).0 as i32
            };

            if selected_index != LB_ERR {
                unsafe {
                    let ptr = SendMessageW(
                        list_box,
                        LB_GETITEMDATA,
                        Some(WPARAM(selected_index as usize)),
                        Some(LPARAM(0)),
                    );
                    let window_title = ptr.0 as *const String;
                    println!("Selected: {}", *window_title);
                    PostQuitMessage(0);
                    return Some(LRESULT(0));
                };
            }
            return Some(LRESULT(0));
        }
    }
    None
}

fn default_window_proc(_hwnd: HWND, msg: u32, wparam: WPARAM, _lparam: LPARAM) -> Option<LRESULT> {
    if let Some(result) = default_exit_on_esc(msg, wparam) {
        return Some(result);
    }

    if msg == WM_DESTROY {
        unsafe { PostQuitMessage(0) };
        return Some(LRESULT(0));
    }

    None
}

extern "system" fn text_box_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        if let Some(result) = default_window_proc(hwnd, msg, wparam, lparam) {
            return result;
        }

        let app_context = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const AppContext;

        if !app_context.is_null() {
            let app_context = &(*app_context);

            if let Some(result) = default_select_on_return(app_context.list_box.hwnd, msg, wparam) {
                return result;
            }

            if let Some(result) =
                route_nav_keys_to_list_box(app_context.list_box.hwnd, msg, wparam, lparam)
            {
                return result;
            }

            if msg == WM_KEYUP {
                let length = GetWindowTextLengthW(hwnd) as usize;
                let open_windows = &app_context.open_windows;

                let items: Vec<&String> = if length > 0 {
                    let mut buffer = vec![0u16; length + 1];
                    GetWindowTextW(hwnd, &mut buffer);

                    let search_string = get_str_from_buffer(&buffer);
                    let mut result: Vec<(&String, i32)> = Vec::with_capacity(open_windows.len());

                    for window_title in open_windows.iter() {
                        let score = fuzzy_compare(&search_string, window_title);
                        if score > 0 {
                            result.push((window_title, score));
                        }
                    }

                    result.sort_by(|a, b| b.1.cmp(&a.1));
                    result.iter().map(|i| i.0).collect()
                } else {
                    open_windows.iter().collect()
                };

                if !items.is_empty() {
                    updated_list_box(app_context.list_box.hwnd, items.into_iter());
                }
            }

            let proc_ptr = app_context.text_box.original_proc_ptr;
            let proc = Some(std::mem::transmute::<
                isize,
                unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT,
            >(proc_ptr));
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
    if let Some(result) = default_exit_on_esc(msg, wparam) {
        return result;
    }

    if let Some(result) = default_select_on_return(hwnd, msg, wparam) {
        return result;
    }

    let app_context = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const AppContext;
    if !app_context.is_null() {
        if let Some(result) =
            route_key_up_to_text_box((*app_context).text_box.hwnd, msg, wparam, lparam)
        {
            return result;
        }

        let proc_ptr = (*app_context).list_box.original_proc_ptr;
        let proc = Some(std::mem::transmute::<
            isize,
            unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT,
        >(proc_ptr));
        CallWindowProcW(proc, hwnd, msg, wparam, lparam)
    } else {
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

fn fuzzy_compare(search: &str, to_search: &str) -> i32 {
    let mut score: i32 = 0;

    for (i, c) in search.char_indices() {
        if let Some(r) = to_search.find(|ct: char| ct.eq_ignore_ascii_case(&c)) {
            if r == i {
                score += 2;
            } else {
                score += 1;
            }
        }
    }
    score
}

fn get_str_from_buffer(buffer_with_null_ter: &[u16]) -> String {
    String::from_utf16_lossy(&buffer_with_null_ter[0..buffer_with_null_ter.len() - 1])
}

unsafe fn updated_list_box<'a, T>(list_box: HWND, windows: T)
where
    T: Iterator<Item = &'a String>,
{
    SendMessageW(list_box, LB_RESETCONTENT, Some(WPARAM(0)), Some(LPARAM(0)));

    for (i, window_title) in windows.enumerate() {
        let window_ptr: isize = window_title as *const String as _;

        let title_wide = wide_null(std::ffi::OsStr::new(window_title));
        SendMessageW(
            list_box,
            LB_ADDSTRING,
            Some(WPARAM(0)),
            Some(LPARAM(title_wide.as_ptr() as isize)),
        );

        SendMessageW(
            list_box,
            LB_SETITEMDATA,
            Some(WPARAM(i)),
            Some(LPARAM(window_ptr)),
        );
    }

    SendMessageW(list_box, LB_SETCURSEL, Some(WPARAM(0)), Some(LPARAM(0)));
}
