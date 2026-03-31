#![allow(dead_code)]

use windows::core::{Error, Result};
use windows::Win32::Graphics::Direct2D::{
    Common::D2D1_COLOR_F, D2D1ConvertColorSpace, D2D1_COLOR_SPACE,
};

fn call_d2_d1_convert_color_space() -> Result<D2D1_COLOR_F> {
    let source = D2D1_COLOR_SPACE(0);
    let dest = D2D1_COLOR_SPACE(1);
    let color = D2D1_COLOR_F {
        r: 1.0,
        g: 0.5,
        b: 0.2,
        a: 1.0,
    };

    // SAFETY: D2D1ConvertColorSpace is a pure math conversion function.
    // We pass a valid pointer to a locally constructed `D2D1_COLOR_F` instance.
    let result = unsafe { D2D1ConvertColorSpace(source, dest, &color) };
    Ok(result)
}
