use std::ptr::null;
use windows::core::w;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};

use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, EndPaint, GetStockObject, COLOR_WINDOWFRAME, GET_STOCK_OBJECT_FLAGS, HBRUSH, HFONT, PAINTSTRUCT
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;

use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW, GetSystemMetrics, GetWindowLongPtrW, GetWindowTextLengthW, GetWindowTextW, LoadCursorW, PostQuitMessage, RegisterClassW, SendMessageW, SetForegroundWindow, SetWindowLongPtrW, SetWindowPos, ShowWindow, TranslateMessage, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, GWL_WNDPROC, HWND_TOP, IDC_ARROW, LBS_NOTIFY, LB_ADDSTRING, LB_ERR, LB_GETCURSEL, LB_GETITEMDATA, LB_RESETCONTENT, LB_SETCURSEL, LB_SETITEMDATA, MSG, SM_CXSCREEN, SM_CYSCREEN, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, SW_SHOW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_CHAR, WM_CREATE, WM_DESTROY, WM_KEYDOWN, WM_KEYUP, WM_PAINT, WM_SETFONT, WNDCLASSW, WS_CHILD, WS_POPUP, WS_VISIBLE, WS_VSCROLL
};

use windows::Win32::UI::Input::KeyboardAndMouse::{SetFocus, VIRTUAL_KEY, VK_DOWN, VK_RETURN, VK_UP};

use crate::{tab::WindowInfo, tab};

const VK_ESCAPE: u32 = 0x1B;
const TEXT_BOX_HEIGHT: i32 = 36;


static mut FTAB_SWITCHER: Option<FSwitch> = None;

pub struct FSwitch {
    main_window: HWND,
}

struct UIElement {
    hwnd: HWND,
    original_proc_ptr: isize,
}


struct AppContext {
    text_box: UIElement,
    list_box: UIElement,
    open_windows: Box<Vec<WindowInfo>>

}

impl FSwitch {
    pub fn new() -> FSwitch {

        // get open windows
        let open_windows = Box::new(tab::get_open_windows());

        unsafe {
            if FTAB_SWITCHER.is_some() {
                panic!("there can be only one FSwitch struct!")
            }

            let instance: HINSTANCE = GetModuleHandleW(None).unwrap().into();

            let window_class = w!("window");

            let background = GetStockObject(GET_STOCK_OBJECT_FLAGS(COLOR_WINDOWFRAME.0));

            // Define the window class
            let wnd_class = WNDCLASSW {
                hInstance: instance,
                lpszClassName: window_class,
                lpfnWndProc: Some(window_proc),
                style: CS_HREDRAW | CS_VREDRAW,
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
                // hbrBackground: (HBRUSH)(COLOR_WINDOW),
                hbrBackground: HBRUSH(background.0),
                ..Default::default()
            };

            RegisterClassW(&wnd_class);

            // Create the window
            // pass the open windows with lpparm
            let open_windows_ptr = Box::into_raw(open_windows);

            let hwnd: HWND = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                window_class,
                w!("fSwitch"),
                WS_POPUP | WS_VISIBLE
                , // Hide title bar and other window components
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                600, // Width of the search bar
                400, // Height of the search bar
                None,
                None,
                instance,
                Some(open_windows_ptr as _),
            )
                .unwrap();

            let corner_preference = DWMWCP_ROUND;

            DwmSetWindowAttribute(
                hwnd,
                DWMWA_WINDOW_CORNER_PREFERENCE,
                &corner_preference as *const _ as *const _,
                std::mem::size_of_val(&corner_preference) as u32,
            ).unwrap();

            // Center the window on the screen
            let screen_width = GetSystemMetrics(SM_CXSCREEN);
            let screen_height = GetSystemMetrics(SM_CYSCREEN);
            let window_width = 600;

            SetWindowPos(
                hwnd,
                HWND_TOP,
                (screen_width - window_width) / 2,
                // (screen_height - window_height) / 2,
                (screen_height as f32 * 0.25) as i32,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_SHOWWINDOW,
            )
                .unwrap();


            FSwitch { main_window: hwnd}
        }
    }

    pub fn run(&self) {
        unsafe {
            ShowWindow(self.main_window, SW_SHOW).unwrap();

            let mut msg = MSG::default();
            // msg.lParam = LPARAM(String::from("Hi").as_ptr() as isize);
            while GetMessageW(&mut msg, None, 0, 0).into() {
                _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }



}
extern "system" fn window_proc(main_window: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // run default proc, which should run for every control
    if let Some(result) = default_window_proc(main_window, msg, wparam, lparam) {
        return result;
    }

    // if appcontext is alrady created, run other default procs...
    unsafe {
        let app_context = GetWindowLongPtrW(main_window, GWLP_USERDATA) as *const AppContext;

        if !app_context.is_null() {
            let app_context = &(*app_context);


            if let Some(result) = route_nav_keys_to_list_box(app_context.list_box.hwnd, msg, wparam, lparam) {
                return result;
            }

            if let Some(result) = route_key_up_to_text_box(app_context.text_box.hwnd, msg, wparam, lparam) {
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
                // get open windows from lparm
                let cs = lparam.0 as *const CREATESTRUCTW;
                // let open_windows = (*cs).lpCreateParams as *const Vec<WindowInfo>;
                let open_windows = Box::from_raw((*cs).lpCreateParams as *mut Vec<WindowInfo>);


                // setup child objects...
                let mut rect = windows::Win32::Foundation::RECT::default();
                GetClientRect(main_window, &mut rect).unwrap();

                // child objects
                let text_box = create_text_box(main_window, rect);
                let list_box = create_list_box(main_window, rect);

                let app_context = Box::new(
                    AppContext {
                        list_box,
                        text_box,
                        open_windows,
                    }
                );

                let app_context_ptr = &*app_context as *const AppContext as _;
                // let app_context_ptr = Box::into_raw(app_context) as _;
                SetWindowLongPtrW(main_window, GWLP_USERDATA, app_context_ptr);
                SetWindowLongPtrW(app_context.list_box.hwnd, GWLP_USERDATA, app_context_ptr);
                SetWindowLongPtrW(app_context.text_box.hwnd, GWLP_USERDATA, app_context_ptr);

                // setFont
                let font = create_font();
                SendMessageW(app_context.text_box.hwnd, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1)); // 1 to redraw
                SendMessageW(app_context.list_box.hwnd, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1)); // 1 to redraw

                // random stuff (todo!)
                SetFocus(app_context.text_box.hwnd).unwrap();

                // init the list box, with the current open windows...
                updated_list_box(app_context.list_box.hwnd, app_context.open_windows.iter());

                Box::leak(app_context);
            };

            LRESULT(0)
        }

        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            unsafe {
                let _hdc = BeginPaint(main_window, &mut ps);
                _ = EndPaint(main_window, &ps);
            }
            LRESULT(0)
        }

        _ => unsafe { DefWindowProcW(main_window, msg, wparam, lparam) },
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
        main_window,
        None,
        GetModuleHandleW(None).unwrap(),
        None,
    ).unwrap();


    let original_proc_ptr = SetWindowLongPtrW(text_box, GWL_WNDPROC, text_box_proc as isize);

    UIElement {
        hwnd: text_box,
        original_proc_ptr
    }
}

unsafe fn create_list_box(main_window: HWND, rect: RECT) -> UIElement {

    let list_box = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        w!("ListBox"),
        w!(""),
        WS_CHILD | WS_VISIBLE | WINDOW_STYLE(LBS_NOTIFY.try_into().unwrap()) | WS_VSCROLL,
        0,
        TEXT_BOX_HEIGHT, // Start under the textbox
        rect.right,
        rect.bottom - TEXT_BOX_HEIGHT, // Height adjusted to half of the window
        main_window,
        None,
        GetModuleHandleW(None).unwrap(),
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
        -24,                // Height (negative value for character height)
        0,                   // Width
        0,              // Escapement
        0,             // Orientation
        400,                // Weight (e.g., 400 for normal, 700 for bold)
        0,                  // Italic
        0,               // Underline
        0,               // StrikeOut
        0,                 // CharSet (DEFAULT_CHARSET)
        0,            // OutPrecision
        0,           // ClipPrecision
        0,                 // Quality
        0,          // Pitch and Family
        w!("Segoe UI"), // Font face name
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

fn route_key_up_to_text_box(text_box: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
    if msg == WM_KEYUP || msg == WM_CHAR {

        let key = wparam.0 as u32;
        let vkey = VIRTUAL_KEY(key as u16);

        //ignore RETURN
        if vkey == VK_RETURN {
            return None;
        }

        // ignore nav keys...
        if vkey == VK_UP || vkey == VK_DOWN {
            return None;
        }

        unsafe  {
            SetFocus(text_box).unwrap();
            return Some(SendMessageW(text_box, msg, wparam, lparam));
        }
    }

    None
}

fn route_nav_keys_to_list_box(list_box: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
    if msg == WM_KEYDOWN {
        let key = wparam.0 as u32;
        let vkey = VIRTUAL_KEY(key as u16);


        if vkey == VK_UP || vkey == VK_DOWN {
            unsafe {
                SetFocus(list_box).unwrap();
                return Some(SendMessageW(list_box, msg, wparam, lparam));
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
            let selected_index = unsafe { SendMessageW(list_box, LB_GETCURSEL, WPARAM(0), LPARAM(0)).0 as i32 };

            if selected_index != LB_ERR {
                unsafe {
                    let ptr = SendMessageW(list_box
                                           ,LB_GETITEMDATA
                                           ,WPARAM(selected_index as usize)
                                           ,LPARAM(0)
                    );

                    let window= ptr.0 as *const WindowInfo;
                    let result = SetForegroundWindow((*window).hwnd);

                    if result.as_bool() {
                        // exit app
                        PostQuitMessage(0);
                    }
                    return Some(LRESULT(0));
                };
            }

            return Some(LRESULT(0));
        }
    }

    None
}

fn default_window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {

    // run default on exit
    if let Some(result) = default_exit_on_esc(msg, wparam) {
        return Some(result);
    }

    let app_context = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const AppContext};

    let app_context = if app_context.is_null() {
        None
    }else {
        unsafe {
            Some(&(*app_context))
        }
    };

    match msg {
        WM_KEYUP => {
            let key = wparam.0 as u32;
            let vkey = VIRTUAL_KEY(key as u16);
            if vkey == VK_DOWN {
                unsafe {
                    if let Some(context) = app_context {
                        SetFocus(context.list_box.hwnd).unwrap();
                        SendMessageW(context.list_box.hwnd, LB_SETCURSEL, WPARAM(1), LPARAM(0));
                    }
                }

                return Some(LRESULT(0));
            }
        }

        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            return Some(LRESULT(0));
        }

        _ => {}
    }

    None
}


extern "system" fn text_box_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        // run default proc, which should run for every control
        if let Some(result) = default_window_proc(hwnd, msg, wparam, lparam) {
            return result;
        }

        let app_context = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const AppContext;

        if !app_context.is_null() {

            let app_context = &(*app_context);

            if let Some(result) = default_select_on_return(app_context.list_box.hwnd, msg, wparam) {
                return result;
            }

            if let Some(result) = route_nav_keys_to_list_box(app_context.list_box.hwnd, msg, wparam, lparam) {
                return result;
            }

            if msg == WM_KEYUP {
                let length = GetWindowTextLengthW(hwnd) as usize;
                let open_windows = &app_context.open_windows;

                let items =
                    if length > 0 {
                        let mut buffer = vec![0u16; length + 1];
                        GetWindowTextW(hwnd, &mut buffer);


                        let search_string = get_str_from_buffer(&buffer);
                        let mut result: Vec<(&WindowInfo, i32)> = Vec::with_capacity(open_windows.len());

                        for window in open_windows.iter() {
                            let window_title = get_str_from_buffer(&window.title);
                            let score = fuzzy_compare(&search_string, &window_title);

                            if score > 0 {
                                result.push((window, score));
                            }
                        }

                        result.sort_by(|a, b| b.1.cmp(&a.1));


                        let result: Vec<&WindowInfo> = result.iter()
                            .map(|i| i.0)
                            .collect();

                        result
                    }else {
                        let result = open_windows.iter().collect();
                        result
                    };

                if !items.is_empty() {
                    updated_list_box(app_context.list_box.hwnd, items.into_iter());
                }
            }


            let proc_ptr = app_context.text_box.original_proc_ptr;
            let proc = Some(std::mem::transmute(proc_ptr));

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
    // run default on exit
    if let Some(result) = default_exit_on_esc(msg, wparam) {
        return result;
    }

    if let Some(result) = default_select_on_return(hwnd, msg, wparam) {
        return result;
    }


    let app_context = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const AppContext;
    if app_context != null() {
        if let Some(result) = route_key_up_to_text_box((*app_context).text_box.hwnd, msg, wparam, lparam) {
            return result;
        }

        let proc_ptr = (*app_context).list_box.original_proc_ptr;
        let proc = Some(std::mem::transmute(proc_ptr));

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
            }else {
                score += 1;
            }
        }
    }

    score
}

fn get_str_from_buffer(buffer_with_null_ter: &Vec<u16>) -> String {
    //exclude the \0 terminator
    String::from_utf16_lossy(&buffer_with_null_ter[0.. buffer_with_null_ter.len() - 1])
}

unsafe fn updated_list_box<'a, T>(list_box: HWND, windows: T)
where
    T: Iterator<Item = &'a WindowInfo>,
{

    // clear list box
    SendMessageW(list_box, LB_RESETCONTENT, WPARAM(0), LPARAM(0));

    let mut i = 0;

    // add new items...
    for window in windows {
        let window_ptr: isize = window as *const WindowInfo as _;

        SendMessageW(
            list_box,
            LB_ADDSTRING,
            WPARAM(0),
            LPARAM(
                window.title.as_ptr() as isize,
            ),
        );

        SendMessageW(list_box
                     ,LB_SETITEMDATA
                     ,WPARAM(i)
                     ,LPARAM(window_ptr)
        );

        i += 1;
    }

    // auto select first item
    SendMessageW(
        list_box,
        LB_SETCURSEL,
        WPARAM(0),
        LPARAM(0),
    );
}