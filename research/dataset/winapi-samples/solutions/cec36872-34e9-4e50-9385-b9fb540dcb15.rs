use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{E_FAIL, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassExW, SendMessageW, ShowWindow, TranslateMessage, CW_USEDEFAULT, LBS_NOTIFY,
    LB_ADDSTRING, LB_SETCURSEL, MSG, SW_SHOW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY,
    WNDCLASSEXW, WS_BORDER, WS_CHILD, WS_VISIBLE, WS_VSCROLL,
};

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?.into();
        let window_class_name = wide_null("ListBoxSelectionExample");

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wndproc),
            hInstance: instance,
            lpszClassName: PCWSTR(window_class_name.as_ptr()),
            ..Default::default()
        };

        if RegisterClassExW(&wc) == 0 {
            return Err(Error::from_thread());
        }

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(window_class_name.as_ptr()),
            PCWSTR(wide_null("ListBox Selection Example").as_ptr()),
            WINDOW_STYLE(0),
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            Some(instance),
            None,
        )?;

        ShowWindow(hwnd, SW_SHOW);

        let listbox_class = wide_null("ListBox");
        let listbox = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(listbox_class.as_ptr()),
            PCWSTR::null(),
            WINDOW_STYLE(
                WS_CHILD.0 | WS_VISIBLE.0 | WS_BORDER.0 | WS_VSCROLL.0 | LBS_NOTIFY as u32,
            ),
            10,
            10,
            200,
            200,
            Some(hwnd),
            None,
            Some(instance),
            None,
        )?;

        // Add items to the listbox
        let items = ["Apple", "Banana", "Cherry", "Date", "Elderberry"];
        for item in items {
            let wide_item = wide_null(item);
            let result = SendMessageW(
                listbox,
                LB_ADDSTRING,
                Some(WPARAM(0)),
                Some(LPARAM(wide_item.as_ptr() as isize)),
            );
            if result.0 == -1 {
                return Err(Error::from_thread());
            }
        }

        // Select the third item (index 2) using LB_SETCURSEL
        let selection_index = 2;
        let result = SendMessageW(
            listbox,
            LB_SETCURSEL,
            Some(WPARAM(selection_index)),
            Some(LPARAM(0)),
        );
        if result.0 == -1 {
            // LB_SETCURSEL returns -1 if the index is out of range
            return Err(Error::from_hresult(E_FAIL));
        }

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
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
