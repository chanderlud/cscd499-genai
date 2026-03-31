use windows::core::HRESULT;
use windows::Win32::Graphics::Direct2D::{
    Common::D2D1_COLOR_F, D2D1ConvertColorSpace, D2D1_COLOR_SPACE,
};

fn call_d2_d1_convert_color_space() -> HRESULT {
    let color = D2D1_COLOR_F {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    let _result =
        unsafe { D2D1ConvertColorSpace(D2D1_COLOR_SPACE(0), D2D1_COLOR_SPACE(1), &color) };
    HRESULT(0)
}
