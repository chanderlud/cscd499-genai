// TITLE: Handle non-client area messages to center text vertically in edit control

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::*;

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

struct EditControl {
    hwnd: HWND,
    background_brush: HBRUSH,
    subclass_id: usize,
}

impl EditControl {
    fn new(parent: HWND, text: &str, x: i32, y: i32, width: i32, height: i32) -> Result<Self> {
        let class_name = wide_null("EDIT".as_ref());
        let window_text = wide_null(text.as_ref());

        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                PCWSTR(class_name.as_ptr()),
                PCWSTR(window_text.as_ptr()),
                WS_CHILD | WS_VISIBLE | WS_BORDER | WINDOW_STYLE(ES_AUTOHSCROLL as u32),
                x,
                y,
                width,
                height,
                Some(parent),
                None,
                Some(GetModuleHandleW(None)?.into()),
                None,
            )
        }?;

        let background_brush = unsafe { CreateSolidBrush(COLORREF(0xC8C8C8)) }; // Light gray

        let mut control = Self {
            hwnd,
            background_brush,
            subclass_id: 1,
        };

        control.setup_subclass()?;
        Ok(control)
    }

    fn setup_subclass(&mut self) -> Result<()> {
        unsafe {
            let success = SetWindowSubclass(
                self.hwnd,
                Some(Self::subclass_proc),
                self.subclass_id,
                Box::into_raw(Box::new(self.background_brush)) as usize,
            );
            if !success.as_bool() {
                return Err(Error::from_thread());
            }
        }
        Ok(())
    }

    unsafe extern "system" fn subclass_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        _uid_ref: usize,
        dw_ref_data: usize,
    ) -> LRESULT {
        match msg {
            WM_NCCALCSIZE => {
                if wparam.0 != 0 {
                    // Calculate client area height needed for the font
                    let font_handle =
                        SendMessageW(hwnd, WM_GETFONT, Some(WPARAM(0)), Some(LPARAM(0)));
                    let font_handle = HGDIOBJ(font_handle.0 as _);
                    let mut text_rect: RECT = std::mem::zeroed();
                    let hdc = GetDC(Some(hwnd));

                    let old_font = SelectObject(hdc, font_handle);
                    let mut calc_text: [u16; 2] = [75, 121]; // "Wy" characters for height calculation
                    DrawTextW(hdc, &mut calc_text, &mut text_rect, DT_CALCRECT | DT_LEFT);

                    let text_height = text_rect.bottom - text_rect.top;
                    SelectObject(hdc, old_font);
                    ReleaseDC(Some(hwnd), hdc);

                    // Calculate NC area to center text vertically
                    let mut client_rect: RECT = std::mem::zeroed();
                    let mut window_rect: RECT = std::mem::zeroed();
                    GetClientRect(hwnd, &mut client_rect);
                    GetWindowRect(hwnd, &mut window_rect);

                    let window_height = window_rect.bottom - window_rect.top;
                    let center_offset = ((window_height - text_height) / 2) - 4;

                    // Adjust the client area
                    let info_ptr = lparam.0 as *mut NCCALCSIZE_PARAMS;
                    let info = &mut *info_ptr;
                    info.rgrc[0].top += center_offset;
                    info.rgrc[0].bottom -= center_offset;
                }
                LRESULT(0)
            }
            WM_NCPAINT => {
                let brush = HBRUSH(dw_ref_data as *mut _);

                let mut window_rect: RECT = std::mem::zeroed();
                let mut client_rect: RECT = std::mem::zeroed();
                GetWindowRect(hwnd, &mut window_rect);
                GetClientRect(hwnd, &mut client_rect);

                let mut top_left = POINT {
                    x: window_rect.left,
                    y: window_rect.top,
                };
                ScreenToClient(hwnd, &mut top_left);

                let mut bottom_right = POINT {
                    x: window_rect.right,
                    y: window_rect.bottom,
                };
                ScreenToClient(hwnd, &mut bottom_right);

                let top_rect = RECT {
                    left: 0,
                    top: top_left.y,
                    right: client_rect.right,
                    bottom: client_rect.top,
                };

                let bottom_rect = RECT {
                    left: 0,
                    top: client_rect.bottom,
                    right: client_rect.right,
                    bottom: bottom_right.y,
                };

                let hdc = GetDC(Some(hwnd));
                FillRect(hdc, &top_rect, brush);
                FillRect(hdc, &bottom_rect, brush);
                ReleaseDC(Some(hwnd), hdc);

                LRESULT(0)
            }
            WM_SIZE => {
                // Force non-client area repaint
                let _ = SetWindowPos(
                    hwnd,
                    None,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOOWNERZORDER | SWP_NOSIZE | SWP_NOMOVE | SWP_FRAMECHANGED,
                );
                DefSubclassProc(hwnd, msg, wparam, lparam)
            }
            _ => DefSubclassProc(hwnd, msg, wparam, lparam),
        }
    }
}

impl Drop for EditControl {
    fn drop(&mut self) {
        unsafe {
            RemoveWindowSubclass(self.hwnd, Some(Self::subclass_proc), self.subclass_id);
            DeleteObject(self.background_brush.into());
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

fn main() -> Result<()> {
    // Create a simple window to host the edit control
    let class_name = wide_null("MainWindowClass".as_ref());
    let window_title = wide_null("Edit Control with Centered Text".as_ref());

    unsafe {
        let instance = GetModuleHandleW(None)?;

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hbrBackground: GetSysColorBrush(COLOR_WINDOW),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        let atom = RegisterClassExW(&wc);
        if atom == 0 {
            return Err(Error::from_thread());
        }

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_title.as_ptr()),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            200,
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        // Create the edit control with custom non-client area handling
        let _edit = EditControl::new(hwnd, "Centered Text", 50, 50, 200, 30)?;

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
