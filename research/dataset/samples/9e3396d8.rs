use windows::core::w;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, GetWindowLongPtrW,
    PostQuitMessage, RegisterClassW, SetWindowLongPtrW, ShowWindow, TranslateMessage,
    CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, MSG, SW_SHOW,
    WINDOW_EX_STYLE, WM_CREATE, WM_DESTROY, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

struct WindowData {
    message: String,
}

extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            unsafe {
                // Extract data passed via lpCreateParams
                let cs = lparam.0 as *const CREATESTRUCTW;
                let data_ptr = (*cs).lpCreateParams as *mut WindowData;
                let data = Box::from_raw(data_ptr);

                // Store data in window's user data for later access
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(data) as _);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe {
                // Clean up stored data
                let data_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowData;
                if !data_ptr.is_null() {
                    let _ = Box::from_raw(data_ptr);
                }
                PostQuitMessage(0);
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

fn main() -> windows::core::Result<()> {
    unsafe {
        let instance: HINSTANCE = GetModuleHandleW(None)?.into();
        let window_class = w!("DataPassingWindow");

        let wnd_class = WNDCLASSW {
            hInstance: instance,
            lpszClassName: window_class,
            lpfnWndProc: Some(window_proc),
            style: CS_HREDRAW | CS_VREDRAW,
            ..Default::default()
        };

        RegisterClassW(&wnd_class);

        // Create data to pass to window procedure
        let window_data = Box::new(WindowData {
            message: String::from("Hello from lpCreateParams!"),
        });

        // Pass data pointer via lpCreateParams
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            window_class,
            w!("Data Passing Example"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            Some(instance),
            Some(Box::into_raw(window_data) as _),
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
