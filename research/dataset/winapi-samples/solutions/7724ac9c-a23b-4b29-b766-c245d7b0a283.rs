use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassW, SendMessageW, TranslateMessage, BS_PUSHBUTTON, CS_HREDRAW, CS_VREDRAW,
    CW_USEDEFAULT, MSG, WINDOW_STYLE, WM_DESTROY, WM_GETTEXT, WM_GETTEXTLENGTH, WM_SETTEXT,
    WNDCLASSW, WS_CHILD, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

fn main() -> Result<()> {
    unsafe {
        let class_name = wide_null("SampleWindowClass");
        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            hInstance: Default::default(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        if RegisterClassW(&wc) == 0 {
            return Err(Error::from_thread());
        }

        let window_title = wide_null("Button Text Example");
        let hwnd = CreateWindowExW(
            Default::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_title.as_ptr()),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            None,
            None,
        )?;

        let button_class = wide_null("BUTTON");
        let button_text = wide_null("Initial Text");
        let button_hwnd = CreateWindowExW(
            Default::default(),
            PCWSTR(button_class.as_ptr()),
            PCWSTR(button_text.as_ptr()),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_PUSHBUTTON as u32),
            50,
            50,
            200,
            30,
            Some(hwnd),
            None,
            None,
            None,
        )?;

        // Get current button text length
        let length = SendMessageW(
            button_hwnd,
            WM_GETTEXTLENGTH,
            Some(WPARAM(0)),
            Some(LPARAM(0)),
        )
        .0;
        println!("Initial text length: {}", length);

        // Get current button text
        let mut buffer = vec![0u16; (length + 1) as usize];
        let chars_copied = SendMessageW(
            button_hwnd,
            WM_GETTEXT,
            Some(WPARAM(buffer.len())),
            Some(LPARAM(buffer.as_mut_ptr() as isize)),
        )
        .0;
        let text = String::from_utf16_lossy(&buffer[..chars_copied as usize]);
        println!("Initial text: {}", text);

        // Set new button text
        let new_text = wide_null("Updated Text");
        let result = SendMessageW(
            button_hwnd,
            WM_SETTEXT,
            Some(WPARAM(0)),
            Some(LPARAM(new_text.as_ptr() as isize)),
        );
        if result.0 == 0 {
            return Err(Error::from_thread());
        }

        // Verify new text was set
        let new_length = SendMessageW(
            button_hwnd,
            WM_GETTEXTLENGTH,
            Some(WPARAM(0)),
            Some(LPARAM(0)),
        )
        .0;
        println!("New text length: {}", new_length);

        let mut new_buffer = vec![0u16; (new_length + 1) as usize];
        let new_chars_copied = SendMessageW(
            button_hwnd,
            WM_GETTEXT,
            Some(WPARAM(new_buffer.len())),
            Some(LPARAM(new_buffer.as_mut_ptr() as isize)),
        )
        .0;
        let new_text_result = String::from_utf16_lossy(&new_buffer[..new_chars_copied as usize]);
        println!("New text: {}", new_text_result);

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
