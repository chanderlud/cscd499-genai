use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HMODULE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassExW, SendMessageW, ShowWindow, TranslateMessage, CW_USEDEFAULT, LB_ADDSTRING,
    LB_ERR, LB_GETCOUNT, MSG, SW_SHOWDEFAULT, WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY,
    WNDCLASSEXW, WS_BORDER, WS_CHILD, WS_VISIBLE, WS_VSCROLL,
};

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

fn main() -> Result<()> {
    // Register window class
    let class_name = wide_null("ListBoxCountExample");
    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        lpfnWndProc: Some(wndproc),
        // SAFETY: GetModuleHandleW is safe to call with None
        hInstance: unsafe { GetModuleHandleW(None)? }.into(),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        ..Default::default()
    };

    // SAFETY: RegisterClassExW is safe to call with valid WNDCLASSEXW
    let atom = unsafe { RegisterClassExW(&wc) };
    if atom == 0 {
        return Err(Error::from_thread());
    }

    // Create main window
    // SAFETY: CreateWindowExW is safe with valid parameters
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(wide_null("ListBox Count Example").as_ptr()),
            WINDOW_STYLE(WS_VISIBLE.0),
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            None,
            None,
        )?
    };

    // Create ListBox control
    let listbox_class = wide_null("ListBox");
    // SAFETY: CreateWindowExW is safe with valid parameters
    let listbox = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(listbox_class.as_ptr()),
            PCWSTR::null(),
            WINDOW_STYLE((WS_CHILD | WS_VISIBLE | WS_VSCROLL | WS_BORDER).0),
            10,
            10,
            200,
            200,
            Some(hwnd),
            None,
            None,
            None,
        )?
    };

    // Add items to ListBox
    let items = ["Apple", "Banana", "Cherry", "Date", "Elderberry"];
    for item in items.iter() {
        let wide_item = wide_null(item);
        // SAFETY: SendMessageW is safe with valid parameters
        let result = unsafe {
            SendMessageW(
                listbox,
                LB_ADDSTRING,
                Some(WPARAM(0)),
                Some(LPARAM(wide_item.as_ptr() as isize)),
            )
        };
        if result.0 as i32 == LB_ERR {
            return Err(Error::from_thread());
        }
    }

    // Get the count of items using LB_GETCOUNT
    // SAFETY: SendMessageW is safe with valid parameters
    let count = unsafe { SendMessageW(listbox, LB_GETCOUNT, Some(WPARAM(0)), Some(LPARAM(0))) };

    if count.0 as i32 == LB_ERR {
        return Err(Error::from_thread());
    }

    println!("Number of items in ListBox: {}", count.0);

    // Show window and run message loop
    // SAFETY: ShowWindow is safe with valid HWND
    unsafe { ShowWindow(hwnd, SW_SHOWDEFAULT) };
    let mut msg = MSG::default();
    // SAFETY: GetMessageW is safe with valid MSG pointer
    while unsafe { GetMessageW(&mut msg, None, 0, 0) }.into() {
        // SAFETY: TranslateMessage and DispatchMessageW are safe with valid MSG
        unsafe {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    Ok(())
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => {
            // SAFETY: PostQuitMessage is safe to call
            unsafe {
                PostQuitMessage(0);
            }
            LRESULT(0)
        }
        // SAFETY: DefWindowProcW is safe with valid parameters
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
