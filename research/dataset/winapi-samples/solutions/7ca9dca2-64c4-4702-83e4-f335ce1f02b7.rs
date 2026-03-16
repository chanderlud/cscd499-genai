use std::ffi::OsStr;
use std::iter::once;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassExW, SendMessageW, TranslateMessage, CW_USEDEFAULT, LB_ADDSTRING, LB_SELECTSTRING,
    MSG, WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY, WNDCLASSEXW,
};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        // Register window class
        let class_name = wide_null(OsStr::new("ListBoxSearchExample"));
        let wc = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wndproc),
            hInstance: GetModuleHandleW(None)?.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };
        RegisterClassExW(&wc);

        // Create main window
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(wide_null(OsStr::new("ListBox Search Example")).as_ptr()),
            WINDOW_STYLE(0x00CF0000), // WS_OVERLAPPEDWINDOW
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            None,
            None,
        )?;

        // Create ListBox control
        let listbox_class = wide_null(OsStr::new("ListBox"));
        let listbox = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(listbox_class.as_ptr()),
            PCWSTR::null(),
            WINDOW_STYLE(0x50010001), // WS_VISIBLE | WS_CHILD | WS_BORDER | WS_VSCROLL | LBS_NOTIFY
            10,
            10,
            360,
            200,
            Some(hwnd),
            None,
            None,
            None,
        )?;

        // Add items to ListBox
        let items = [
            "Apple",
            "Banana",
            "Cherry",
            "Date",
            "Elderberry",
            "Fig",
            "Grape",
        ];
        for item in items {
            let wide_item = wide_null(OsStr::new(item));
            SendMessageW(
                listbox,
                LB_ADDSTRING,
                Some(WPARAM(0)),
                Some(LPARAM(wide_item.as_ptr() as isize)),
            );
        }

        // Search for and select item starting with "Ch"
        let search_text = wide_null(OsStr::new("Ch"));
        let result = SendMessageW(
            listbox,
            LB_SELECTSTRING,
            Some(WPARAM(0)), // Start search from beginning
            Some(LPARAM(search_text.as_ptr() as isize)),
        );

        if result.0 == -1 {
            println!("No item found starting with 'Ch'");
        } else {
            println!("Selected item at index: {}", result.0);
        }

        // Message loop
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Ok(())
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
