use windows::core::{w, Result, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINTS, RECT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetParent, GetWindowLongPtrW, GetWindowRect,
    PostMessageW, RegisterClassExW, SetWindowLongPtrW, SetWindowPos, CREATESTRUCTW, GWLP_USERDATA,
    GWL_STYLE, HTBOTTOM, HTBOTTOMLEFT, HTBOTTOMRIGHT, HTLEFT, HTRIGHT, HTTOP, HTTOPLEFT,
    HTTOPRIGHT, HTTRANSPARENT, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOOWNERZORDER,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_CREATE, WM_NCDESTROY, WM_NCHITTEST, WM_NCLBUTTONDOWN,
    WNDCLASSEXW, WS_CHILD, WS_CLIPSIBLINGS, WS_SIZEBOX, WS_VISIBLE,
};

const CLIENT: isize = 0b0000;
const LEFT: isize = 0b0001;
const RIGHT: isize = 0b0010;
const TOP: isize = 0b0100;
const BOTTOM: isize = 0b1000;
const TOPLEFT: isize = TOP | LEFT;
const TOPRIGHT: isize = TOP | RIGHT;
const BOTTOMLEFT: isize = BOTTOM | LEFT;
const BOTTOMRIGHT: isize = BOTTOM | RIGHT;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum HitTestResult {
    Client,
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    NoWhere,
}

impl HitTestResult {
    fn to_win32(self) -> i32 {
        match self {
            HitTestResult::Left => HTLEFT as _,
            HitTestResult::Right => HTRIGHT as _,
            HitTestResult::Top => HTTOP as _,
            HitTestResult::Bottom => HTBOTTOM as _,
            HitTestResult::TopLeft => HTTOPLEFT as _,
            HitTestResult::TopRight => HTTOPRIGHT as _,
            HitTestResult::BottomLeft => HTBOTTOMLEFT as _,
            HitTestResult::BottomRight => HTBOTTOMRIGHT as _,
            _ => HTTRANSPARENT,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn hit_test(
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    cx: i32,
    cy: i32,
    border_x: i32,
    border_y: i32,
) -> HitTestResult {
    #[rustfmt::skip]
    let result = (LEFT * (cx < left + border_x) as isize)
               | (RIGHT * (cx >= right - border_x) as isize)
               | (TOP * (cy < top + border_y) as isize)
               | (BOTTOM * (cy >= bottom - border_y) as isize);

    match result {
        CLIENT => HitTestResult::Client,
        LEFT => HitTestResult::Left,
        RIGHT => HitTestResult::Right,
        TOP => HitTestResult::Top,
        BOTTOM => HitTestResult::Bottom,
        TOPLEFT => HitTestResult::TopLeft,
        TOPRIGHT => HitTestResult::TopRight,
        BOTTOMLEFT => HitTestResult::BottomLeft,
        BOTTOMRIGHT => HitTestResult::BottomRight,
        _ => HitTestResult::NoWhere,
    }
}

const CLASS_NAME: PCWSTR = w!("TAURI_DRAG_RESIZE_BORDERS");
const WINDOW_NAME: PCWSTR = w!("TAURI_DRAG_RESIZE_WINDOW");

struct UndecoratedResizingData {
    child: HWND,
    has_undecorated_shadows: bool,
}

unsafe extern "system" fn drag_resize_window_proc(
    child: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let data = lparam.0 as *mut CREATESTRUCTW;
            let data = (*data).lpCreateParams as *mut UndecoratedResizingData;
            (*data).child = child;
            SetWindowLongPtrW(child, GWLP_USERDATA, data as _);
        }

        WM_NCHITTEST => {
            let data = GetWindowLongPtrW(child, GWLP_USERDATA);
            let data = &*(data as *mut UndecoratedResizingData);

            let Ok(parent) = GetParent(child) else {
                return DefWindowProcW(child, msg, wparam, lparam);
            };
            let style = GetWindowLongPtrW(parent, GWL_STYLE);
            let style = WINDOW_STYLE(style as u32);

            let is_resizable = (style & WS_SIZEBOX).0 != 0;
            if !is_resizable {
                return DefWindowProcW(child, msg, wparam, lparam);
            }

            if data.has_undecorated_shadows {
                return LRESULT(HTTOP as _);
            }

            let mut rect = RECT::default();
            if GetWindowRect(child, &mut rect).is_err() {
                return DefWindowProcW(child, msg, wparam, lparam);
            }

            let (cx, cy) = (GET_X_LPARAM(lparam) as i32, GET_Y_LPARAM(lparam) as i32);

            let border_x = 8; // Fixed border width for simplicity
            let border_y = 8; // Fixed border height for simplicity

            let res = hit_test(
                rect.left,
                rect.top,
                rect.right,
                rect.bottom,
                cx,
                cy,
                border_x,
                border_y,
            );

            return LRESULT(res.to_win32() as _);
        }

        WM_NCLBUTTONDOWN => {
            let data = GetWindowLongPtrW(child, GWLP_USERDATA);
            let data = &*(data as *mut UndecoratedResizingData);

            let Ok(parent) = GetParent(child) else {
                return DefWindowProcW(child, msg, wparam, lparam);
            };
            let style = GetWindowLongPtrW(parent, GWL_STYLE);
            let style = WINDOW_STYLE(style as u32);

            let is_resizable = (style & WS_SIZEBOX).0 != 0;
            if !is_resizable {
                return DefWindowProcW(child, msg, wparam, lparam);
            }

            let (cx, cy) = (GET_X_LPARAM(lparam) as i32, GET_Y_LPARAM(lparam) as i32);

            let res = if data.has_undecorated_shadows {
                HitTestResult::Top
            } else {
                let mut rect = RECT::default();
                if GetWindowRect(child, &mut rect).is_err() {
                    return DefWindowProcW(child, msg, wparam, lparam);
                }

                let border_x = 8;
                let border_y = 8;

                hit_test(
                    rect.left,
                    rect.top,
                    rect.right,
                    rect.bottom,
                    cx,
                    cy,
                    border_x,
                    border_y,
                )
            };

            if res != HitTestResult::NoWhere {
                let points = POINTS {
                    x: cx as i16,
                    y: cy as i16,
                };

                let _ = PostMessageW(
                    Some(parent),
                    WM_NCLBUTTONDOWN,
                    WPARAM(res.to_win32() as _),
                    LPARAM(&points as *const _ as _),
                );
            }

            return LRESULT(0);
        }

        WM_NCDESTROY => {
            let data = GetWindowLongPtrW(child, GWLP_USERDATA);
            let data = data as *mut UndecoratedResizingData;
            drop(Box::from_raw(data));
        }

        _ => {}
    }

    DefWindowProcW(child, msg, wparam, lparam)
}

#[allow(non_snake_case)]
#[inline]
fn GET_X_LPARAM(lparam: LPARAM) -> i16 {
    ((lparam.0 as usize) & 0xFFFF) as u16 as i16
}

#[allow(non_snake_case)]
#[inline]
fn GET_Y_LPARAM(lparam: LPARAM) -> i16 {
    (((lparam.0 as usize) & 0xFFFF_0000) >> 16) as u16 as i16
}

fn main() -> Result<()> {
    unsafe {
        let class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(drag_resize_window_proc),
            lpszClassName: CLASS_NAME,
            ..Default::default()
        };

        let atom = RegisterClassExW(&class);
        if atom == 0 {
            return Err(windows::core::Error::from_thread());
        }

        let data = Box::new(UndecoratedResizingData {
            child: HWND::default(),
            has_undecorated_shadows: false,
        });

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            CLASS_NAME,
            WINDOW_NAME,
            WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS,
            0,
            0,
            100,
            100,
            None,
            None,
            Some(HINSTANCE(GetModuleHandleW(None)?.0)),
            Some(Box::into_raw(data) as _),
        )?;

        // Simulate a resize message
        let _ = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            200,
            200,
            SWP_ASYNCWINDOWPOS | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
        );

        // Clean up
        DestroyWindow(hwnd)?;

        Ok(())
    }
}
