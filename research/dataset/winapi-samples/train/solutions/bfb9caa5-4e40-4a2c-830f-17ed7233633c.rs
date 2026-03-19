use std::collections::HashMap;
use std::sync::Mutex;
use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmEnableBlurBehindWindow, DWM_BB_ENABLE, DWM_BLURBEHIND};
use windows::Win32::Graphics::Gdi::{ClientToScreen, ScreenToClient};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{ReleaseCapture, SetCapture};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetWindowLongW, GetWindowRect,
    RegisterClassExW, SetWindowLongW, SetWindowPos, ShowWindow, GWL_EXSTYLE, HWND_TOPMOST,
    SWP_NOACTIVATE, SW_SHOW, WINDOW_EX_STYLE, WM_DESTROY, WNDCLASSEXW, WS_EX_LAYERED,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResizeDirection {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

struct DragResizeState {
    is_dragging: bool,
    is_resizing: bool,
    resize_direction: Option<ResizeDirection>,
    start_point: POINT,
    start_rect: RECT,
}

static STATE_MAP: Mutex<Option<HashMap<isize, DragResizeState>>> = Mutex::new(None);

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

fn with_state<F, R>(hwnd: HWND, f: F) -> Result<R>
where
    F: FnOnce(&mut DragResizeState) -> Result<R>,
{
    let mut guard = STATE_MAP
        .lock()
        .map_err(|_| Error::from_hresult(HRESULT::from_win32(14)))?;
    let map = guard.get_or_insert_with(HashMap::new);
    let key = hwnd.0 as isize;
    let state = map.entry(key).or_insert_with(|| DragResizeState {
        is_dragging: false,
        is_resizing: false,
        resize_direction: None,
        start_point: POINT { x: 0, y: 0 },
        start_rect: RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
    });
    f(state)
}

fn remove_state(hwnd: HWND) -> Result<()> {
    let mut map = STATE_MAP
        .lock()
        .map_err(|_| Error::from_hresult(HRESULT::from_win32(14)))?;
    if let Some(map) = map.as_mut() {
        map.remove(&(hwnd.0 as isize));
    }
    Ok(())
}

pub fn client_to_screen(hwnd: HWND, point: &POINT) -> POINT {
    let mut pt = *point;
    unsafe {
        // Convert client coordinates to screen coordinates
        let _ = ClientToScreen(hwnd, &mut pt);
    }
    pt
}

pub fn screen_to_client(hwnd: HWND, point: &POINT) -> POINT {
    let mut pt = *point;
    unsafe {
        // Convert screen coordinates to client coordinates
        let _ = ScreenToClient(hwnd, &mut pt);
    }
    pt
}

pub fn create_draggable_overlay(class_name: &str, initial_rect: &RECT) -> Result<HWND> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        // Convert HMODULE to HINSTANCE
        let hinstance = HINSTANCE(instance.0);
        let class_name_wide = wide_null(class_name);

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinstance,
            lpszClassName: PCWSTR(class_name_wide.as_ptr()),
            ..Default::default()
        };

        let atom = RegisterClassExW(&wc);
        if atom == 0 {
            return Err(Error::from_thread());
        }

        let ex_style = WINDOW_EX_STYLE(
            WS_EX_LAYERED.0 | WS_EX_TOPMOST.0 | WS_EX_TOOLWINDOW.0 | WS_EX_TRANSPARENT.0,
        );

        let hwnd = CreateWindowExW(
            ex_style,
            PCWSTR(class_name_wide.as_ptr()),
            PCWSTR(wide_null("Overlay").as_ptr()),
            WS_POPUP,
            initial_rect.left,
            initial_rect.top,
            initial_rect.right - initial_rect.left,
            initial_rect.bottom - initial_rect.top,
            None,
            None,
            Some(hinstance),
            None,
        )?;

        // Enable blur behind for transparency effect
        let blur_behind = DWM_BLURBEHIND {
            dwFlags: DWM_BB_ENABLE,
            fEnable: true.into(),
            hRgnBlur: Default::default(),
            fTransitionOnMaximized: false.into(),
        };
        let _ = DwmEnableBlurBehindWindow(hwnd, &blur_behind);

        // Show the window
        let _ = ShowWindow(hwnd, SW_SHOW);

        // Initialize state
        with_state(hwnd, |_| Ok(()))?;

        Ok(hwnd)
    }
}

pub fn hit_test_resize(hwnd: HWND, screen_point: &POINT) -> Option<ResizeDirection> {
    unsafe {
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return None;
        }

        let border = 8;
        let x = screen_point.x;
        let y = screen_point.y;

        let in_left = x >= rect.left && x < rect.left + border;
        let in_right = x <= rect.right && x > rect.right - border;
        let in_top = y >= rect.top && y < rect.top + border;
        let in_bottom = y <= rect.bottom && y > rect.bottom - border;

        if in_top && in_left {
            Some(ResizeDirection::NorthWest)
        } else if in_top && in_right {
            Some(ResizeDirection::NorthEast)
        } else if in_bottom && in_left {
            Some(ResizeDirection::SouthWest)
        } else if in_bottom && in_right {
            Some(ResizeDirection::SouthEast)
        } else if in_left {
            Some(ResizeDirection::West)
        } else if in_right {
            Some(ResizeDirection::East)
        } else if in_top {
            Some(ResizeDirection::North)
        } else if in_bottom {
            Some(ResizeDirection::South)
        } else {
            None
        }
    }
}

pub fn begin_drag(hwnd: HWND, screen_point: &POINT) -> Result<()> {
    with_state(hwnd, |state| {
        state.is_dragging = true;
        state.start_point = *screen_point;

        unsafe {
            GetWindowRect(hwnd, &mut state.start_rect)?;

            // Remove WS_EX_TRANSPARENT to make window interactive
            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            let new_ex_style = WINDOW_EX_STYLE(ex_style as u32 & !WS_EX_TRANSPARENT.0);
            SetWindowLongW(hwnd, GWL_EXSTYLE, new_ex_style.0 as i32);

            // Capture mouse to receive events even outside window
            SetCapture(hwnd);
        }

        Ok(())
    })
}

pub fn begin_resize(hwnd: HWND, screen_point: &POINT, direction: ResizeDirection) -> Result<()> {
    with_state(hwnd, |state| {
        state.is_resizing = true;
        state.resize_direction = Some(direction);
        state.start_point = *screen_point;

        unsafe {
            GetWindowRect(hwnd, &mut state.start_rect)?;

            // Remove WS_EX_TRANSPARENT to make window interactive
            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            let new_ex_style = WINDOW_EX_STYLE(ex_style as u32 & !WS_EX_TRANSPARENT.0);
            SetWindowLongW(hwnd, GWL_EXSTYLE, new_ex_style.0 as i32);

            // Capture mouse to receive events even outside window
            SetCapture(hwnd);
        }

        Ok(())
    })
}

pub fn update_drag_resize(hwnd: HWND, screen_point: &POINT) -> Result<()> {
    with_state(hwnd, |state| {
        if state.is_dragging {
            let dx = screen_point.x - state.start_point.x;
            let dy = screen_point.y - state.start_point.y;

            let new_left = state.start_rect.left + dx;
            let new_top = state.start_rect.top + dy;
            let width = state.start_rect.right - state.start_rect.left;
            let height = state.start_rect.bottom - state.start_rect.top;

            unsafe {
                let _ = SetWindowPos(
                    hwnd,
                    Some(HWND_TOPMOST),
                    new_left,
                    new_top,
                    width,
                    height,
                    SWP_NOACTIVATE,
                );
            }
        } else if state.is_resizing {
            let dx = screen_point.x - state.start_point.x;
            let dy = screen_point.y - state.start_point.y;

            let mut new_rect = state.start_rect;

            match state.resize_direction {
                Some(ResizeDirection::North) => {
                    new_rect.top += dy;
                }
                Some(ResizeDirection::South) => {
                    new_rect.bottom += dy;
                }
                Some(ResizeDirection::East) => {
                    new_rect.right += dx;
                }
                Some(ResizeDirection::West) => {
                    new_rect.left += dx;
                }
                Some(ResizeDirection::NorthEast) => {
                    new_rect.top += dy;
                    new_rect.right += dx;
                }
                Some(ResizeDirection::NorthWest) => {
                    new_rect.top += dy;
                    new_rect.left += dx;
                }
                Some(ResizeDirection::SouthEast) => {
                    new_rect.bottom += dy;
                    new_rect.right += dx;
                }
                Some(ResizeDirection::SouthWest) => {
                    new_rect.bottom += dy;
                    new_rect.left += dx;
                }
                None => {}
            }

            // Ensure window has minimum size
            let min_size = 1;
            if new_rect.right - new_rect.left < min_size {
                match state.resize_direction {
                    Some(ResizeDirection::West)
                    | Some(ResizeDirection::NorthWest)
                    | Some(ResizeDirection::SouthWest) => {
                        new_rect.left = new_rect.right - min_size;
                    }
                    _ => {
                        new_rect.right = new_rect.left + min_size;
                    }
                }
            }
            if new_rect.bottom - new_rect.top < min_size {
                match state.resize_direction {
                    Some(ResizeDirection::North)
                    | Some(ResizeDirection::NorthWest)
                    | Some(ResizeDirection::NorthEast) => {
                        new_rect.top = new_rect.bottom - min_size;
                    }
                    _ => {
                        new_rect.bottom = new_rect.top + min_size;
                    }
                }
            }

            unsafe {
                let width = new_rect.right - new_rect.left;
                let height = new_rect.bottom - new_rect.top;
                let _ = SetWindowPos(
                    hwnd,
                    Some(HWND_TOPMOST),
                    new_rect.left,
                    new_rect.top,
                    width,
                    height,
                    SWP_NOACTIVATE,
                );
            }
        }

        Ok(())
    })
}

pub fn end_drag_resize(hwnd: HWND) -> Result<()> {
    with_state(hwnd, |state| {
        state.is_dragging = false;
        state.is_resizing = false;
        state.resize_direction = None;

        unsafe {
            // Release mouse capture
            ReleaseCapture();

            // Restore WS_EX_TRANSPARENT for click-through
            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            let new_ex_style = WINDOW_EX_STYLE(ex_style as u32 | WS_EX_TRANSPARENT.0);
            SetWindowLongW(hwnd, GWL_EXSTYLE, new_ex_style.0 as i32);
        }

        Ok(())
    })
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY => {
            let _ = remove_state(hwnd);
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
