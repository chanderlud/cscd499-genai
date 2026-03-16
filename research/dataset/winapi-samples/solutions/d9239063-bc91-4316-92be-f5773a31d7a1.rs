use windows::core::{w, Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINTS, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CombineRgn, CreateRectRgn, DeleteObject, SetWindowRgn, RGN_DIFF,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect, GetParent, GetWindowLongPtrW,
    GetWindowPlacement, GetWindowRect, PostMessageW, RegisterClassExW, SetWindowLongPtrW,
    SetWindowPos, CREATESTRUCTW, GWLP_USERDATA, GWL_STYLE, HTBOTTOM, HTBOTTOMLEFT, HTBOTTOMRIGHT,
    HTLEFT, HTRIGHT, HTTOP, HTTOPLEFT, HTTOPRIGHT, HTTRANSPARENT, HWND_TOP, SWP_ASYNCWINDOWPOS,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER, SW_MAXIMIZE, WINDOWPLACEMENT, WINDOW_EX_STYLE,
    WINDOW_STYLE, WM_CREATE, WM_NCDESTROY, WM_NCHITTEST, WM_NCLBUTTONDOWN, WM_SIZE, WM_USER,
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

fn is_maximized(window: HWND) -> Result<bool> {
    let mut placement = WINDOWPLACEMENT {
        length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
        ..WINDOWPLACEMENT::default()
    };
    // SAFETY: GetWindowPlacement is safe to call with valid window handle and properly initialized struct
    unsafe { GetWindowPlacement(window, &mut placement)? };
    Ok(placement.showCmd == SW_MAXIMIZE.0 as u32)
}

#[allow(non_snake_case)]
fn GET_X_LPARAM(lparam: LPARAM) -> i16 {
    ((lparam.0 as usize) & 0xFFFF) as u16 as i16
}

#[allow(non_snake_case)]
fn GET_Y_LPARAM(lparam: LPARAM) -> i16 {
    (((lparam.0 as usize) & 0xFFFF_0000) >> 16) as u16 as i16
}

unsafe fn set_drag_hwnd_rgn(hwnd: HWND, width: i32, height: i32, only_top: bool) {
    let border_x = 8; // Fixed border width for example
    let border_y = 8; // Fixed border height for example

    // SAFETY: CreateRectRgn is safe to call with valid coordinates
    let mut hrgn1 = CreateRectRgn(0, 0, width, height);

    let x1 = if only_top { 0 } else { border_x };
    let y1 = border_y;
    let x2 = if only_top { width } else { width - border_x };
    let y2 = if only_top { height } else { height - border_y };

    // SAFETY: CreateRectRgn is safe to call with valid coordinates
    let hrgn2 = CreateRectRgn(x1, y1, x2, y2);

    // SAFETY: CombineRgn is safe to call with valid region handles
    CombineRgn(Some(hrgn1), Some(hrgn1), Some(hrgn2), RGN_DIFF);

    // SAFETY: SetWindowRgn is safe to call with valid window handle and region
    if SetWindowRgn(hwnd, Some(hrgn1), true) == 0 {
        // If it fails, we must free hrgn1 manually
        // SAFETY: DeleteObject is safe to call with valid region handle
        let _ = DeleteObject(hrgn1.into());
    }
}

unsafe extern "system" fn subclass_parent(
    parent: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _: usize,
    data: usize,
) -> LRESULT {
    match msg {
        WM_SIZE => {
            let data = data as *mut UndecoratedResizingData;
            let data = &*data;
            let child = data.child;
            let has_undecorated_shadows = data.has_undecorated_shadows;

            if is_maximized(parent).unwrap_or(false) {
                let _ = SetWindowPos(
                    child,
                    Some(HWND_TOP),
                    0,
                    0,
                    0,
                    0,
                    SWP_ASYNCWINDOWPOS | SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOMOVE,
                );
            } else {
                let mut rect = RECT::default();
                if GetClientRect(parent, &mut rect).is_ok() {
                    let width = rect.right - rect.left;
                    let height = rect.bottom - rect.top;

                    let _ = SetWindowPos(
                        child,
                        Some(HWND_TOP),
                        0,
                        0,
                        width,
                        height,
                        SWP_ASYNCWINDOWPOS | SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOMOVE,
                    );

                    set_drag_hwnd_rgn(child, width, height, has_undecorated_shadows);
                }
            }
        }

        WM_NCDESTROY => {
            let data = data as *mut UndecoratedResizingData;
            drop(Box::from_raw(data));
        }

        _ => {}
    }

    DefSubclassProc(parent, msg, wparam, lparam)
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

            let border_x = 8; // Fixed border width for example
            let border_y = 8; // Fixed border height for example

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

                let border_x = 8; // Fixed border width for example
                let border_y = 8; // Fixed border height for example

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

fn main() -> Result<()> {
    let class = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: Default::default(),
        lpfnWndProc: Some(drag_resize_window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: unsafe { GetModuleHandleW(None)?.into() },
        hIcon: Default::default(),
        hCursor: Default::default(),
        hbrBackground: Default::default(),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: CLASS_NAME,
        hIconSm: Default::default(),
    };

    // SAFETY: RegisterClassExW is safe to call with valid class struct
    unsafe { RegisterClassExW(&class) };

    let data = UndecoratedResizingData {
        child: HWND::default(),
        has_undecorated_shadows: false,
    };

    // SAFETY: CreateWindowExW is safe to call with valid parameters
    let drag_window = unsafe {
        CreateWindowExW(
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
            GetModuleHandleW(None).ok().map(|h| h.into()),
            Some(Box::into_raw(Box::new(data)) as _),
        )
    }?;

    // SAFETY: SetWindowSubclass is safe to call with valid window handle and subclass proc
    unsafe {
        let data = UndecoratedResizingData {
            child: drag_window,
            has_undecorated_shadows: false,
        };

        let _ = SetWindowSubclass(
            drag_window,
            Some(subclass_parent),
            (WM_USER + 1) as _,
            Box::into_raw(Box::new(data)) as _,
        );
    }

    // SAFETY: DestroyWindow is safe to call with valid window handle
    unsafe { DestroyWindow(drag_window)? };

    Ok(())
}
