use windows::core::w;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, EndPaint, GetStockObject, COLOR_WINDOWFRAME, GET_STOCK_OBJECT_FLAGS,
    HBRUSH, PAINTSTRUCT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW, LoadCursorW,
    PostQuitMessage, RegisterClassW, SendMessageW, ShowWindow, TranslateMessage, CS_HREDRAW,
    CS_VREDRAW, CW_USEDEFAULT, IDC_ARROW, MSG, SW_SHOW, WM_CREATE, WM_DESTROY, WM_PAINT,
    WM_SETFONT, WNDCLASSW, WS_CHILD, WS_POPUP, WS_VISIBLE,
};

fn main() -> windows::core::Result<()> {
    unsafe {
        let instance: HINSTANCE = GetModuleHandleW(None)?.into();
        let window_class = w!("EditControlExample");

        let background = GetStockObject(GET_STOCK_OBJECT_FLAGS(COLOR_WINDOWFRAME.0));

        let wnd_class = WNDCLASSW {
            hInstance: instance,
            lpszClassName: window_class,
            lpfnWndProc: Some(window_proc),
            style: CS_HREDRAW | CS_VREDRAW,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hbrBackground: HBRUSH(background.0),
            ..Default::default()
        };

        RegisterClassW(&wnd_class);

        let hwnd = CreateWindowExW(
            Default::default(),
            window_class,
            w!("Edit Control Example"),
            WS_POPUP | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            Some(instance),
            None,
        )?;

        let _ = ShowWindow(hwnd, SW_SHOW);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Ok(())
    }
}

extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            unsafe {
                let mut rect = RECT::default();
                GetClientRect(hwnd, &mut rect).unwrap();

                let edit_control = CreateWindowExW(
                    Default::default(),
                    w!("Edit"),
                    w!(""),
                    WS_CHILD | WS_VISIBLE,
                    10,
                    10,
                    rect.right - 20,
                    30,
                    Some(hwnd),
                    None,
                    Some(GetModuleHandleW(None).unwrap().into()),
                    None,
                )
                .unwrap();

                let font = CreateFontW(
                    -24,
                    0,
                    0,
                    0,
                    400,
                    0,
                    0,
                    0,
                    windows::Win32::Graphics::Gdi::FONT_CHARSET(0),
                    windows::Win32::Graphics::Gdi::FONT_OUTPUT_PRECISION(0),
                    windows::Win32::Graphics::Gdi::FONT_CLIP_PRECISION(0),
                    windows::Win32::Graphics::Gdi::FONT_QUALITY(0),
                    0,
                    w!("Segoe UI"),
                );

                SendMessageW(
                    edit_control,
                    WM_SETFONT,
                    Some(WPARAM(font.0 as usize)),
                    Some(LPARAM(1)),
                );
            }
            LRESULT(0)
        }
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            unsafe {
                let _hdc = BeginPaint(hwnd, &mut ps);
                _ = EndPaint(hwnd, &ps);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
