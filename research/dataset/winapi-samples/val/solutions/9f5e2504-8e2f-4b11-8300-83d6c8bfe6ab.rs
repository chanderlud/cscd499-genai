use std::mem;
use std::sync::Once;
use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, WIN32_ERROR, WPARAM};
use windows::Win32::Graphics::Gdi::HBRUSH;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetWindowLongPtrW, LoadCursorW, PostQuitMessage,
    RegisterClassExW, SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowPos, CS_HREDRAW,
    CS_VREDRAW, GWLP_USERDATA, GWL_EXSTYLE, HTCLIENT, HTTRANSPARENT, HWND_TOPMOST, IDC_ARROW,
    LWA_ALPHA, SET_WINDOW_POS_FLAGS, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SWP_NOZORDER, SWP_SHOWWINDOW, WINDOW_EX_STYLE, WM_DESTROY, WM_NCHITTEST, WNDCLASSEXW,
    WNDCLASS_STYLES, WS_EX_LAYERED, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
};

static REGISTER_CLASS: Once = Once::new();
static mut CLASS_ATOM: u16 = 0;

// Helper to convert OsStr to null-terminated UTF-16
fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

// Window state stored in GWLP_USERDATA
#[repr(C)]
struct OverlayState {
    click_through: bool,
    opacity: u8,
}

// Window procedure
unsafe extern "system" fn overlay_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_NCHITTEST => {
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const OverlayState;
            if !state_ptr.is_null() {
                let state = &*state_ptr;
                if state.click_through {
                    return LRESULT(HTTRANSPARENT as isize);
                }
            }
            LRESULT(HTCLIENT as isize)
        }
        WM_DESTROY => {
            // Clean up state
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverlayState;
            if !state_ptr.is_null() {
                drop(Box::from_raw(state_ptr));
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

pub fn create_toggle_overlay(class_name: &str, width: i32, height: i32) -> Result<HWND> {
    let class_name_wide = wide_null(std::ffi::OsStr::new(class_name));

    // Register window class once
    REGISTER_CLASS.call_once(|| {
        let wc = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
            style: WNDCLASS_STYLES(CS_HREDRAW.0 | CS_VREDRAW.0),
            lpfnWndProc: Some(overlay_wndproc),
            hInstance: HINSTANCE(unsafe { GetModuleHandleW(None).unwrap().0 }),
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW).unwrap() },
            hbrBackground: HBRUSH((5i32 + 1) as *mut std::ffi::c_void), // COLOR_WINDOW + 1
            lpszClassName: PCWSTR(class_name_wide.as_ptr()),
            ..Default::default()
        };

        let atom = unsafe { RegisterClassExW(&wc) };
        if atom == 0 {
            panic!("Failed to register window class");
        }
        unsafe { CLASS_ATOM = atom };
    });

    // Create window state
    let state = Box::new(OverlayState {
        click_through: false,
        opacity: 255,
    });
    let state_ptr = Box::into_raw(state);

    // Create layered, topmost, popup window
    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST,
            PCWSTR(class_name_wide.as_ptr()),
            PCWSTR(class_name_wide.as_ptr()),
            WS_POPUP,
            0,
            0,
            width,
            height,
            None,
            None,
            Some(HINSTANCE(unsafe { GetModuleHandleW(None).unwrap().0 })),
            Some(state_ptr as *const _),
        )
    }?;

    // Set initial opacity
    unsafe {
        SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA)?;
    }

    // Show window without activating
    unsafe {
        SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SET_WINDOW_POS_FLAGS(SWP_NOMOVE.0 | SWP_NOSIZE.0 | SWP_NOACTIVATE.0 | SWP_SHOWWINDOW.0),
        )?;
    }

    Ok(hwnd)
}

pub fn set_click_through(hwnd: HWND, click_through: bool) -> Result<()> {
    let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverlayState };
    if state_ptr.is_null() {
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_INVALID_WINDOW_HANDLE.0,
        )));
    }

    let state = unsafe { &mut *state_ptr };
    state.click_through = click_through;

    // Update extended style
    let ex_style = unsafe { WINDOW_EX_STYLE(GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32) };
    let new_ex_style = if click_through {
        ex_style | WS_EX_TRANSPARENT
    } else {
        ex_style & !WS_EX_TRANSPARENT
    };

    unsafe {
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex_style.0 as isize);
        // Force window to update without changing position/size/activation
        SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            SET_WINDOW_POS_FLAGS(
                SWP_NOMOVE.0
                    | SWP_NOSIZE.0
                    | SWP_NOZORDER.0
                    | SWP_NOACTIVATE.0
                    | SWP_FRAMECHANGED.0,
            ),
        )?;
    }

    Ok(())
}

pub fn set_overlay_opacity(hwnd: HWND, opacity: u8) -> Result<()> {
    let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverlayState };
    if state_ptr.is_null() {
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_INVALID_WINDOW_HANDLE.0,
        )));
    }

    let state = unsafe { &mut *state_ptr };
    state.opacity = opacity;

    unsafe {
        SetLayeredWindowAttributes(hwnd, COLORREF(0), opacity, LWA_ALPHA)?;
    }

    Ok(())
}

pub fn is_click_through(hwnd: HWND) -> Result<bool> {
    let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const OverlayState };
    if state_ptr.is_null() {
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_INVALID_WINDOW_HANDLE.0,
        )));
    }

    let state = unsafe { &*state_ptr };
    Ok(state.click_through)
}

pub fn get_overlay_opacity(hwnd: HWND) -> Result<u8> {
    let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const OverlayState };
    if state_ptr.is_null() {
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_INVALID_WINDOW_HANDLE.0,
        )));
    }

    let state = unsafe { &*state_ptr };
    Ok(state.opacity)
}

const ERROR_INVALID_WINDOW_HANDLE: WIN32_ERROR = WIN32_ERROR(1400);
