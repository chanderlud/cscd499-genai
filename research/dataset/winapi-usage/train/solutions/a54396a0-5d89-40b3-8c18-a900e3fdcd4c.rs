use windows::Win32::Foundation::{ERROR_SUCCESS, WIN32_ERROR};
use windows::Win32::Graphics::Gdi::RGBTRIPLE;
use windows::Win32::UI::ColorSystem::CMCheckColorsInGamut;

fn call_cm_check_colors_in_gamut() -> WIN32_ERROR {
    // Create dummy RGBTRIPLE data for the API call
    let rgb = RGBTRIPLE {
        rgbtBlue: 0,
        rgbtGreen: 0,
        rgbtRed: 0,
    };

    // Create result buffer
    let mut result: u8 = 0;

    // Call CMCheckColorsInGamut with concrete parameter values
    // hcmtransform: 0 (null handle for this test)
    // lpargbtriple: pointer to RGBTRIPLE
    // lparesult: pointer to result buffer
    // ncount: 1 (single color to check)
    let success = unsafe { CMCheckColorsInGamut(0, &rgb, &mut result, 1) };

    // Convert BOOL result to WIN32_ERROR
    // BOOL is i32, FALSE = 0, TRUE = non-zero
    if success.0 == 0 {
        // Failed - get error code from GetLastError()
        WIN32_ERROR::from_error(&windows::core::Error::from_thread())
            .expect("Failed to convert thread error to WIN32_ERROR")
    } else {
        // Success
        ERROR_SUCCESS
    }
}
