use std::mem::zeroed;
use std::ptr::null_mut;
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{
    COLORREF, FALSE, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM,
};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, EndPaint,
    GetDC, ReleaseDC, SelectObject, AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, BLENDFUNCTION, DIB_RGB_COLORS, HBITMAP, HDC, HGDIOBJ, PAINTSTRUCT, SRCCOPY,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetWindowLongPtrW, KillTimer, PostQuitMessage, RegisterClassW,
    SetTimer, SetWindowLongPtrW, ShowWindow, UpdateLayeredWindow, CS_HREDRAW, CS_VREDRAW,
    GWLP_USERDATA, SW_SHOW, ULW_ALPHA, WM_DESTROY, WM_PAINT, WM_SIZE, WM_TIMER, WNDCLASSW,
    WS_EX_LAYERED, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

const TIMER_ID: usize = 1;
const TIMER_INTERVAL_MS: u32 = 16; // ~60 FPS

struct OverlayData {
    source_rect: RECT,
    overlay_width: i32,
    overlay_height: i32,
    hdc_mem: HDC,
    hbitmap: HBITMAP,
    old_bitmap: HGDIOBJ,
}

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_SIZE => {
            let data_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverlayData;
            if !data_ptr.is_null() {
                let data = &mut *data_ptr;
                let width = (lparam.0 & 0xFFFF) as i32;
                let height = ((lparam.0 >> 16) & 0xFFFF) as i32;
                data.overlay_width = width;
                data.overlay_height = height;
                let _ = update_magnifier(hwnd, data);
            }
            LRESULT(0)
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);
            if hdc.is_invalid() {
                return LRESULT(0);
            }
            let data_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverlayData;
            if !data_ptr.is_null() {
                let data = &*data_ptr;
                let _ = update_magnifier(hwnd, data);
            }
            let result = EndPaint(hwnd, &ps);
            if result == FALSE {
                return LRESULT(0);
            }
            LRESULT(0)
        }
        WM_TIMER => {
            if wparam.0 == TIMER_ID {
                let data_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverlayData;
                if !data_ptr.is_null() {
                    let data = &*data_ptr;
                    let _ = update_magnifier(hwnd, data);
                }
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            let data_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverlayData;
            if !data_ptr.is_null() {
                let data = Box::from_raw(data_ptr);
                let _ = KillTimer(Some(hwnd), TIMER_ID);
                let _ = DeleteObject(data.old_bitmap);
                let _ = DeleteObject(data.hbitmap.into());
                let _ = DeleteDC(data.hdc_mem);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn update_magnifier(hwnd: HWND, data: &OverlayData) -> Result<()> {
    let hdc_screen = GetDC(None);
    if hdc_screen.is_invalid() {
        return Err(Error::from_thread());
    }

    let src_width = data.source_rect.right - data.source_rect.left;
    let src_height = data.source_rect.bottom - data.source_rect.top;

    // Capture screen region
    BitBlt(
        data.hdc_mem,
        0,
        0,
        src_width,
        src_height,
        Some(hdc_screen),
        data.source_rect.left,
        data.source_rect.top,
        SRCCOPY,
    )?;

    // Update layered window with stretched content
    let blend = BLENDFUNCTION {
        BlendOp: AC_SRC_OVER as u8,
        BlendFlags: 0,
        SourceConstantAlpha: 255,
        AlphaFormat: AC_SRC_ALPHA as u8,
    };

    let point = POINT { x: 0, y: 0 };
    let size = SIZE {
        cx: data.overlay_width,
        cy: data.overlay_height,
    };

    UpdateLayeredWindow(
        hwnd,
        None,               // hdcdst: None for layered windows
        None,               // pptdst: None (no position change)
        Some(&size),        // psize: New size
        Some(data.hdc_mem), // hdcsrc: Our memory DC with captured content
        Some(&point),       // ppsrc: Source position
        COLORREF(0),        // crkey: No color key
        Some(&blend),       // pblend: Blend function
        ULW_ALPHA,          // dwflags: Use alpha blending
    )?;

    ReleaseDC(None, hdc_screen);

    Ok(())
}

pub fn create_magnifier_overlay(
    source_rect: &RECT,
    overlay_width: i32,
    overlay_height: i32,
) -> Result<HWND> {
    unsafe {
        let hinstance = GetModuleHandleW(None)?;
        let class_name = wide_null("MagnifierOverlayClass");

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            hInstance: hinstance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        let atom = RegisterClassW(&wc);
        if atom == 0 {
            return Err(Error::from_thread());
        }

        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST,
            PCWSTR(class_name.as_ptr()),
            PCWSTR(wide_null("Magnifier Overlay").as_ptr()),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            0,
            0,
            overlay_width,
            overlay_height,
            None,
            None,
            Some(hinstance.into()),
            None,
        )?;

        // Create memory DC and bitmap for capturing
        let hdc_screen = GetDC(None);
        let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
        let mut bmi: BITMAPINFO = zeroed();
        bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = source_rect.right - source_rect.left;
        bmi.bmiHeader.biHeight = -(source_rect.bottom - source_rect.top); // Top-down
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB.0; // Convert BI_COMPRESSION to u32

        let mut bits: *mut std::ffi::c_void = null_mut();
        let hbitmap = CreateDIBSection(Some(hdc_screen), &bmi, DIB_RGB_COLORS, &mut bits, None, 0)?;
        let old_bitmap = SelectObject(hdc_mem, hbitmap.into());

        ReleaseDC(None, hdc_screen);

        let data = Box::new(OverlayData {
            source_rect: *source_rect,
            overlay_width,
            overlay_height,
            hdc_mem,
            hbitmap,
            old_bitmap,
        });

        let data_ptr = Box::into_raw(data);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, data_ptr as isize);

        // Initial update
        let data_ref = &*data_ptr;
        update_magnifier(hwnd, data_ref)?;

        // Set timer for continuous updates
        SetTimer(Some(hwnd), TIMER_ID, TIMER_INTERVAL_MS, None);

        if ShowWindow(hwnd, SW_SHOW) == FALSE {
            return Err(Error::from_thread());
        }

        Ok(hwnd)
    }
}
