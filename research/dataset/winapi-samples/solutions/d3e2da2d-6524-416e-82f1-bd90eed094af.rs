use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{CreateBitmap, DeleteObject};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassExW, SendMessageW, ShowWindow, TranslateMessage, BM_SETIMAGE, BS_BITMAP,
    IMAGE_BITMAP, MSG, SW_SHOW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY, WNDCLASSEXW,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        // Register window class
        let class_name = wide_null("BitmapButtonExample".as_ref());
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wndproc),
            hInstance: windows::Win32::System::LibraryLoader::GetModuleHandleW(None)?.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };
        RegisterClassExW(&wc);

        // Create main window
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(wide_null("Bitmap Button Example".as_ref()).as_ptr()),
            WINDOW_STYLE(0x00CF0000), // WS_OVERLAPPEDWINDOW
            100,
            100,
            400,
            300,
            None,
            None,
            None,
            None,
        )?;

        // Create button with bitmap style
        let button_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(wide_null("BUTTON".as_ref()).as_ptr()),
            PCWSTR(wide_null("".as_ref()).as_ptr()),
            WINDOW_STYLE(BS_BITMAP as u32 | 0x50010000), // BS_BITMAP | WS_VISIBLE | WS_CHILD | WS_TABSTOP
            50,
            50,
            120,
            40,
            Some(hwnd),
            None,
            None,
            None,
        )?;

        // Create a simple 16x16 red bitmap
        let width = 16;
        let height = 16;
        let bits: Vec<u32> = vec![0x00FF0000; (width * height) as usize]; // Red pixels (ARGB)
        let hbitmap = CreateBitmap(
            width,
            height,
            1,
            32,
            Some(bits.as_ptr() as *const std::ffi::c_void),
        );

        // Check if CreateBitmap succeeded
        if hbitmap.0.is_null() {
            return Err(Error::from_thread());
        }

        // Set the bitmap on the button
        let _result = SendMessageW(
            button_hwnd,
            BM_SETIMAGE,
            Some(WPARAM(IMAGE_BITMAP.0 as usize)),
            Some(LPARAM(hbitmap.0 as isize)),
        );

        // Show window
        ShowWindow(hwnd, SW_SHOW);

        // Message loop
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Clean up bitmap - convert HBITMAP to HGDIOBJ
        DeleteObject(hbitmap.into()).ok()?;

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
