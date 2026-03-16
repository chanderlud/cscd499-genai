use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{COLOR_WINDOW, HBRUSH};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassExW, SendMessageW, ShowWindow, TranslateMessage, CW_USEDEFAULT, LBS_HASSTRINGS,
    LBS_NOTIFY, LB_ADDSTRING, LB_INITSTORAGE, LB_RESETCONTENT, MSG, SW_SHOWDEFAULT,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_CREATE, WM_DESTROY, WNDCLASSEXW, WS_BORDER, WS_CHILD,
    WS_VISIBLE, WS_VSCROLL,
};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;

        let class_name = wide_null(OsStr::new("ListBoxStorageExample"));
        let window_title = wide_null(OsStr::new("ListBox with LB_INITSTORAGE"));

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: Default::default(),
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: HINSTANCE(instance.0),
            hIcon: Default::default(),
            hCursor: Default::default(),
            hbrBackground: HBRUSH(COLOR_WINDOW.0 as isize as *mut _),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            hIconSm: Default::default(),
        };

        if RegisterClassExW(&wc) == 0 {
            return Err(Error::from_thread());
        }

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_title.as_ptr()),
            WINDOW_STYLE(WS_VISIBLE.0 | WS_CHILD.0),
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            Some(HINSTANCE(instance.0)),
            None,
        )?;

        ShowWindow(hwnd, SW_SHOWDEFAULT);

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
                // Create and populate the listbox when the window is created
                if let Err(e) = create_listbox_with_storage(hwnd) {
                    eprintln!("Failed to create listbox: {}", e);
                }
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

fn create_listbox_with_storage(parent: HWND) -> Result<HWND> {
    unsafe {
        let listbox_class = wide_null(OsStr::new("ListBox"));

        // Combine window styles with listbox styles, casting listbox styles to u32
        let listbox_style = WINDOW_STYLE(
            WS_VISIBLE.0
                | WS_CHILD.0
                | WS_BORDER.0
                | WS_VSCROLL.0
                | (LBS_HASSTRINGS as u32)
                | (LBS_NOTIFY as u32),
        );

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(listbox_class.as_ptr()),
            PCWSTR::null(),
            listbox_style,
            10,
            10,
            380,
            280,
            Some(parent),
            None,
            None,
            None,
        )?;

        // Initialize storage for 1000 items with average string length of 20 characters
        let result = SendMessageW(
            hwnd,
            LB_INITSTORAGE,
            Some(WPARAM(1000)),
            Some(LPARAM(20 * 1000)),
        );

        if result.0 < 0 {
            return Err(Error::from_hresult(HRESULT::from_win32(result.0 as u32)));
        }

        // Clear any existing content
        SendMessageW(hwnd, LB_RESETCONTENT, Some(WPARAM(0)), Some(LPARAM(0)));

        // Add items using the pre-allocated storage
        for i in 0..100 {
            let item_text = format!("Item {}", i);
            let wide_text = wide_null(OsStr::new(&item_text));

            SendMessageW(
                hwnd,
                LB_ADDSTRING,
                Some(WPARAM(0)),
                Some(LPARAM(wide_text.as_ptr() as isize)),
            );
        }

        Ok(hwnd)
    }
}
