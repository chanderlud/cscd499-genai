use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::WC_LISTBOXW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, GetWindow, PostQuitMessage,
    RegisterClassExW, SendMessageW, ShowWindow, TranslateMessage, CW_USEDEFAULT, GW_CHILD,
    LB_ADDSTRING, LB_ERR, LB_GETSELCOUNT, LB_GETSELITEMS, LB_SETSEL, MSG, SW_SHOW, WINDOW_EX_STYLE,
    WINDOW_STYLE, WM_DESTROY, WM_LBUTTONUP, WNDCLASSEXW,
};

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class_name = wide_null("MultiSelectListBoxExample");

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        RegisterClassExW(&wc);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(wide_null("Multi-Select ListBox Example").as_ptr()),
            WINDOW_STYLE(0x00CF0000), // WS_OVERLAPPEDWINDOW
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        let listbox = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            WC_LISTBOXW,
            PCWSTR::null(),
            WINDOW_STYLE(0x50210141), // WS_VISIBLE | WS_CHILD | WS_VSCROLL | LBS_MULTIPLESEL | LBS_NOTIFY
            10,
            10,
            360,
            200,
            Some(hwnd),
            None,
            Some(instance.into()),
            None,
        )?;

        // Add items to the listbox
        for item in &["Apple", "Banana", "Cherry", "Date", "Elderberry"] {
            let wide_item = wide_null(item);
            let _ = SendMessageW(
                listbox,
                LB_ADDSTRING,
                Some(WPARAM(0)),
                Some(LPARAM(wide_item.as_ptr() as isize)),
            );
        }

        // Select multiple items (indices 0, 2, and 4)
        for index in [0, 2, 4] {
            let _ = SendMessageW(listbox, LB_SETSEL, Some(WPARAM(1)), Some(LPARAM(index)));
        }

        ShowWindow(hwnd, SW_SHOW);

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).into() {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }

        Ok(())
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_LBUTTONUP => {
            // Get the listbox handle (first child)
            let listbox = GetWindow(hwnd, GW_CHILD);
            if let Ok(listbox) = listbox {
                // Get count of selected items
                let sel_count =
                    SendMessageW(listbox, LB_GETSELCOUNT, Some(WPARAM(0)), Some(LPARAM(0)));

                if sel_count.0 != LB_ERR as isize {
                    let count = sel_count.0;
                    if count > 0 {
                        // Allocate buffer for selected indices
                        let mut indices = vec![0i32; count as usize];
                        let result = SendMessageW(
                            listbox,
                            LB_GETSELITEMS,
                            Some(WPARAM(count as usize)),
                            Some(LPARAM(indices.as_mut_ptr() as isize)),
                        );

                        if result.0 != LB_ERR as isize {
                            let fetched = result.0;
                            println!("Selected {} items:", fetched);
                            for i in 0..fetched as usize {
                                println!("  Index: {}", indices[i]);
                            }
                        }
                    } else {
                        println!("No items selected");
                    }
                }
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
