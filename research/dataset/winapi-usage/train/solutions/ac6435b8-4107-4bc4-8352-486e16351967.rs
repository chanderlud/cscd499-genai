use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
use windows::Win32::Graphics::Direct2D::{D2D1ConvertColorSpace, D2D1_COLOR_SPACE};

fn call_d2_d1_convert_color_space() -> WIN32_ERROR {
    let color = D2D1_COLOR_F {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    unsafe {
        let _ = D2D1ConvertColorSpace(D2D1_COLOR_SPACE(0), D2D1_COLOR_SPACE(0), &color);
    }
    WIN32_ERROR(0)
}
