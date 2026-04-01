use windows::core::{Error, Result};
use windows::Win32::Graphics::Gdi::RGBTRIPLE;
use windows::Win32::UI::ColorSystem::CMCheckColorsInGamut;

fn call_cm_check_colors_in_gamut() -> windows::core::HRESULT {
    // Create sample RGBTRIPLE
    let rgb = RGBTRIPLE {
        rgbtBlue: 0,
        rgbtGreen: 0,
        rgbtRed: 0,
    };

    // Create result buffer
    let mut result: u8 = 0;

    // Call the API
    let ret = unsafe {
        CMCheckColorsInGamut(
            0, // hcmtransform (null for this example)
            &rgb as *const RGBTRIPLE,
            &mut result as *mut u8,
            1, // ncount
        )
    };

    // Convert BOOL result to HRESULT
    if ret.as_bool() {
        windows::core::HRESULT::from_win32(0) // S_OK
    } else {
        windows::core::HRESULT::from_win32(1) // ERROR_SUCCESS equivalent
    }
}
