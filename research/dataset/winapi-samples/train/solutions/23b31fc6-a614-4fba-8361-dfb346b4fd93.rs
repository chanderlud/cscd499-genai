use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, UpdateWindow, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    SRCCOPY,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetWindowRect, PostQuitMessage, RegisterClassW,
    SetLayeredWindowAttributes, SetTimer, ShowWindow, WindowFromPoint, LWA_COLORKEY, SW_SHOW,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY, WM_PAINT, WM_TIMER, WNDCLASSW, WS_EX_LAYERED,
    WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP, WS_VISIBLE,
};

const SAMPLE_SIZE: i32 = 10;
const TIMER_ID: usize = 1;
const SAMPLE_INTERVAL_MS: u32 = 500;

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

fn calculate_average_color(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<COLORREF> {
    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height, // top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let buffer_size = (width * height * 4) as usize;
    let mut buffer = vec![0u8; buffer_size];

    unsafe {
        let hbitmap = CreateCompatibleBitmap(hdc, width, height);
        if hbitmap.is_invalid() {
            return Err(Error::from_thread());
        }

        let hdc_mem = CreateCompatibleDC(Some(hdc));
        if hdc_mem.is_invalid() {
            let _ = DeleteObject(hbitmap.into());
            return Err(Error::from_thread());
        }

        let old_bitmap = SelectObject(hdc_mem, hbitmap.into());
        let result = BitBlt(hdc_mem, 0, 0, width, height, Some(hdc), x, y, SRCCOPY);
        if let Err(err) = result {
            let _ = SelectObject(hdc_mem, old_bitmap);
            let _ = DeleteObject(hbitmap.into());
            let _ = DeleteDC(hdc_mem);
            return Err(Error::from_hresult(err.code()));
        }

        let lines = GetDIBits(
            hdc_mem,
            hbitmap,
            0,
            height as u32,
            Some(buffer.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        let _ = SelectObject(hdc_mem, old_bitmap);
        let _ = DeleteObject(hbitmap.into());
        let _ = DeleteDC(hdc_mem);

        if lines == 0 {
            return Err(Error::from_thread());
        }
    }

    let mut total_r: u64 = 0;
    let mut total_g: u64 = 0;
    let mut total_b: u64 = 0;
    let pixel_count = (width * height) as u64;

    for chunk in buffer.chunks_exact(4) {
        let b = chunk[0] as u64;
        let g = chunk[1] as u64;
        let r = chunk[2] as u64;
        total_r += r;
        total_g += g;
        total_b += b;
    }

    let avg_r = (total_r / pixel_count) as u8;
    let avg_g = (total_g / pixel_count) as u8;
    let avg_b = (total_b / pixel_count) as u8;

    Ok(COLORREF(
        (avg_b as u32) << 16 | (avg_g as u32) << 8 | avg_r as u32,
    ))
}

fn sample_background_color(hwnd_overlay: HWND) -> Result<COLORREF> {
    let mut overlay_rect = RECT::default();
    unsafe { GetWindowRect(hwnd_overlay, &mut overlay_rect) }?;

    let center_x = overlay_rect.left + (overlay_rect.right - overlay_rect.left) / 2;
    let center_y = overlay_rect.top + (overlay_rect.bottom - overlay_rect.top) / 2;
    let point = POINT {
        x: center_x,
        y: center_y,
    };

    let hwnd_target = unsafe { WindowFromPoint(point) };
    if hwnd_target.is_invalid() || hwnd_target == hwnd_overlay {
        return Ok(COLORREF(0x00000000)); // Fallback to black
    }

    let mut target_rect = RECT::default();
    unsafe { GetWindowRect(hwnd_target, &mut target_rect) }?;

    let sample_x = center_x - target_rect.left - SAMPLE_SIZE / 2;
    let sample_y = center_y - target_rect.top - SAMPLE_SIZE / 2;

    let hdc_target = unsafe { GetDC(Some(hwnd_target)) };
    if hdc_target.is_invalid() {
        return Err(Error::from_thread());
    }

    let result = calculate_average_color(hdc_target, sample_x, sample_y, SAMPLE_SIZE, SAMPLE_SIZE);
    unsafe { ReleaseDC(Some(hwnd_target), hdc_target) }; // Added unsafe block

    result
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_TIMER => {
            if wparam.0 == TIMER_ID {
                match sample_background_color(hwnd) {
                    Ok(color) => {
                        let _ = SetLayeredWindowAttributes(hwnd, color, 0, LWA_COLORKEY);
                    }
                    Err(_) => {
                        let fallback = COLORREF(0x00000000);
                        let _ = SetLayeredWindowAttributes(hwnd, fallback, 0, LWA_COLORKEY);
                    }
                }
            }
            LRESULT(0)
        }
        WM_PAINT => LRESULT(0),
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

pub fn create_adaptive_overlay(width: i32, height: i32) -> Result<HWND> {
    let class_name = wide_null("AdaptiveOverlayClass");
    let hinstance = unsafe { GetModuleHandleW(None) }?;

    let wc = WNDCLASSW {
        lpfnWndProc: Some(wndproc),
        hInstance: hinstance.into(),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        ..Default::default()
    };

    let atom = unsafe { RegisterClassW(&wc) };
    if atom == 0 {
        return Err(Error::from_thread());
    }

    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(WS_EX_LAYERED.0 | WS_EX_TOPMOST.0 | WS_EX_TRANSPARENT.0),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(wide_null("Adaptive Overlay").as_ptr()),
            WINDOW_STYLE(WS_POPUP.0 | WS_VISIBLE.0),
            0,
            0,
            width,
            height,
            None,
            None,
            Some(hinstance.into()),
            None,
        )
    }?;

    if hwnd.is_invalid() {
        return Err(Error::from_thread());
    }

    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd).ok()?;
        let timer_id = SetTimer(Some(hwnd), TIMER_ID, SAMPLE_INTERVAL_MS, None);
        if timer_id == 0 {
            return Err(Error::from_thread());
        }
    }

    Ok(hwnd)
}
