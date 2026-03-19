use std::mem::size_of;
use std::ptr::null_mut;
use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, EndPaint,
    GetDC, ReleaseDC, SelectObject, AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, BLENDFUNCTION, DIB_RGB_COLORS, HBITMAP, HDC, PAINTSTRUCT, RGBQUAD, SRCCOPY,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetCursorPos, GetMessageW,
    GetWindowLongPtrW, KillTimer, PostQuitMessage, RegisterClassExW, SetTimer, SetWindowLongPtrW,
    ShowWindow, TranslateMessage, UpdateLayeredWindow, CREATESTRUCTW, GWLP_USERDATA, MSG, SW_SHOW,
    ULW_ALPHA, WM_CREATE, WM_DESTROY, WM_MOUSEWHEEL, WM_PAINT, WM_TIMER, WNDCLASSEXW,
    WNDCLASS_STYLES, WS_EX_LAYERED, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
};

const TIMER_ID: usize = 1;
const TIMER_INTERVAL_MS: u32 = 16; // ~60 FPS

struct MagnifierData {
    zoom: f32,
    width: i32,
    height: i32,
    hdc_mem: HDC,
    hbitmap: HBITMAP,
    bits: *mut u8,
}

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

fn create_dib_section(hdc: HDC, width: i32, height: i32) -> Result<(HBITMAP, *mut u8)> {
    unsafe {
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0 as u32,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD::default(); 1],
        };

        let mut bits: *mut u8 = null_mut();
        let hbitmap = CreateDIBSection(
            Some(hdc),
            &bmi,
            DIB_RGB_COLORS,
            &mut bits as *mut _ as *mut _,
            None,
            0,
        )?;
        if hbitmap.is_invalid() || bits.is_null() {
            return Err(Error::from_hresult(HRESULT::from_win32(0)));
        }
        Ok((hbitmap, bits))
    }
}

fn capture_screen_region(hdc_dest: HDC, x: i32, y: i32, width: i32, height: i32) -> Result<()> {
    unsafe {
        let hdc_screen = GetDC(None);
        if hdc_screen.is_invalid() {
            return Err(Error::from_thread());
        }

        let result = BitBlt(
            hdc_dest,
            0,
            0,
            width,
            height,
            Some(hdc_screen),
            x,
            y,
            SRCCOPY,
        );
        ReleaseDC(None, hdc_screen);

        result?;
        Ok(())
    }
}

fn update_magnifier_window(hwnd: HWND, data: &MagnifierData) -> Result<()> {
    unsafe {
        let mut cursor_pos = POINT::default();
        GetCursorPos(&mut cursor_pos)?;

        let src_width = (data.width as f32 / data.zoom) as i32;
        let src_height = (data.height as f32 / data.zoom) as i32;

        let src_x = cursor_pos.x - src_width / 2;
        let src_y = cursor_pos.y - src_height / 2;

        // Capture screen region to memory DC
        capture_screen_region(data.hdc_mem, src_x, src_y, src_width, src_height)?;

        // Update layered window
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };

        let window_x = cursor_pos.x - data.width / 2;
        let window_y = cursor_pos.y - data.height / 2;

        let window_pos = POINT {
            x: window_x,
            y: window_y,
        };
        let window_size = SIZE {
            cx: data.width,
            cy: data.height,
        };
        let source_pos = POINT::default();

        UpdateLayeredWindow(
            hwnd,
            None,
            Some(&window_pos as *const POINT),
            Some(&window_size as *const SIZE),
            Some(data.hdc_mem),
            Some(&source_pos as *const POINT),
            COLORREF(0),
            Some(&blend as *const BLENDFUNCTION),
            ULW_ALPHA,
        )?;

        Ok(())
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            let create_struct = &*(lparam.0 as *const CREATESTRUCTW);
            let data_ptr = create_struct.lpCreateParams as *mut MagnifierData;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, data_ptr as isize);

            // Set timer for continuous updates
            SetTimer(Some(hwnd), TIMER_ID, TIMER_INTERVAL_MS, None);
            LRESULT(0)
        }
        WM_DESTROY => {
            let data_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut MagnifierData;
            if !data_ptr.is_null() {
                let data = Box::from_raw(data_ptr);
                let _ = DeleteDC(data.hdc_mem);
                let _ = DeleteObject(data.hbitmap.into());
            }
            KillTimer(Some(hwnd), TIMER_ID);
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_PAINT => {
            let data_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut MagnifierData;
            if !data_ptr.is_null() {
                let _data = &*data_ptr;
                let mut ps = PAINTSTRUCT::default();
                let _hdc = BeginPaint(hwnd, &mut ps);
                // Nothing to paint - layered window handles it
                EndPaint(hwnd, &ps);
            }
            LRESULT(0)
        }
        WM_TIMER => {
            if wparam.0 == TIMER_ID {
                let data_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut MagnifierData;
                if !data_ptr.is_null() {
                    let data = &*data_ptr;
                    let _ = update_magnifier_window(hwnd, data);
                }
            }
            LRESULT(0)
        }
        WM_MOUSEWHEEL => {
            let data_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut MagnifierData;
            if !data_ptr.is_null() {
                let data = &mut *data_ptr;
                let delta = (wparam.0 >> 16) as i16; // HIWORD
                let zoom_change = delta as f32 / 1200.0; // 120 is standard wheel delta

                data.zoom = (data.zoom + zoom_change).clamp(1.0, 10.0);
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

pub fn create_magnifier_overlay(width: i32, height: i32, initial_zoom: f32) -> Result<HWND> {
    unsafe {
        let class_name = wide_null("MagnifierOverlayClass");
        let hinstance = GetModuleHandleW(None)?;

        let wnd_class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wndproc),
            hInstance: HINSTANCE(hinstance.0),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            style: WNDCLASS_STYLES(0),
            ..Default::default()
        };

        let atom = RegisterClassExW(&wnd_class);
        if atom == 0 {
            return Err(Error::from_thread());
        }

        // Create memory DC and DIB section for rendering
        let hdc_screen = GetDC(None);
        let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
        let (hbitmap, bits) = create_dib_section(hdc_mem, width, height)?;
        ReleaseDC(None, hdc_screen);

        let old_bitmap = SelectObject(hdc_mem, hbitmap.into());
        if old_bitmap.is_invalid() {
            let _ = DeleteDC(hdc_mem);
            let _ = DeleteObject(hbitmap.into());
            return Err(Error::from_thread());
        }

        let data = Box::new(MagnifierData {
            zoom: initial_zoom.clamp(1.0, 10.0),
            width,
            height,
            hdc_mem,
            hbitmap,
            bits,
        });

        let data_ptr = Box::into_raw(data);

        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST,
            PCWSTR(class_name.as_ptr()),
            PCWSTR(wide_null("Magnifier").as_ptr()),
            WS_POPUP,
            0,
            0,
            width,
            height,
            None,
            None,
            Some(HINSTANCE(hinstance.0)),
            Some(data_ptr as *const _),
        )?;

        if hwnd.is_invalid() {
            // Clean up on failure
            let _ = Box::from_raw(data_ptr);
            return Err(Error::from_thread());
        }

        ShowWindow(hwnd, SW_SHOW);

        Ok(hwnd)
    }
}

// Message loop to keep the window alive
pub fn run_message_loop() -> Result<()> {
    unsafe {
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Ok(())
    }
}
