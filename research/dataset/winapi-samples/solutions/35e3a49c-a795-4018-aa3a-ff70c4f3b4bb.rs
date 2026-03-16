use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{COLOR_WINDOW, HBRUSH};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassExW, SendMessageW, ShowWindow, TranslateMessage, CW_USEDEFAULT, LBS_MULTIPLESEL,
    LB_ADDSTRING, LB_GETSEL, LB_SETSEL, MSG, SW_SHOW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY,
    WNDCLASSEXW, WS_CHILD, WS_VISIBLE, WS_VSCROLL,
};

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;

        let class_name = wide_null("ListBoxExample");
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            hbrBackground: HBRUSH(COLOR_WINDOW.0 as *mut _),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        RegisterClassExW(&wc);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(wide_null("ListBox LB_SETSEL Example").as_ptr()),
            WINDOW_STYLE(0x00CF0000), // WS_OVERLAPPEDWINDOW
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            Some(HINSTANCE(instance.0)),
            None,
        )?;

        // Create a multi-selection listbox
        let listbox_class = wide_null("ListBox");
        let listbox = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(listbox_class.as_ptr()),
            PCWSTR::null(),
            WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | WS_VSCROLL.0 | (LBS_MULTIPLESEL as u32)),
            10,
            10,
            200,
            200,
            Some(hwnd),
            None,
            Some(HINSTANCE(instance.0)),
            None,
        )?;

        // Add some items to the listbox
        for i in 0..5 {
            let item_text = wide_null(&format!("Item {}", i));
            SendMessageW(
                listbox,
                LB_ADDSTRING,
                Some(WPARAM(0)),
                Some(LPARAM(item_text.as_ptr() as isize)),
            );
        }

        // Select items 1 and 3 using LB_SETSEL
        // LB_SETSEL: wParam = TRUE (select) or FALSE (deselect), lParam = item index
        SendMessageW(listbox, LB_SETSEL, Some(WPARAM(1)), Some(LPARAM(1))); // Select item 1
        SendMessageW(listbox, LB_SETSEL, Some(WPARAM(1)), Some(LPARAM(3))); // Select item 3

        // Verify selection using LB_GETSEL
        for i in 0..5 {
            let selected = SendMessageW(listbox, LB_GETSEL, Some(WPARAM(i)), Some(LPARAM(0)));
            println!("Item {} selected: {}", i, selected.0 != 0);
        }

        // Deselect item 1 using LB_SETSEL
        SendMessageW(listbox, LB_SETSEL, Some(WPARAM(0)), Some(LPARAM(1))); // Deselect item 1

        println!("\nAfter deselecting item 1:");
        for i in 0..5 {
            let selected = SendMessageW(listbox, LB_GETSEL, Some(WPARAM(i)), Some(LPARAM(0)));
            println!("Item {} selected: {}", i, selected.0 != 0);
        }

        ShowWindow(hwnd, SW_SHOW);

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
