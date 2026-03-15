// Extract mouse coordinates from LPARAM using GET_X_LPARAM and GET_Y_LPARAM

use windows::core::Result;
use windows::Win32::Foundation::LPARAM;

/// Implementation of the `GET_X_LPARAM` macro.
#[allow(non_snake_case)]
#[inline]
pub fn GET_X_LPARAM(lparam: LPARAM) -> i16 {
    ((lparam.0 as usize) & 0xFFFF) as u16 as i16
}

/// Implementation of the `GET_Y_LPARAM` macro.
#[allow(non_snake_case)]
#[inline]
pub fn GET_Y_LPARAM(lparam: LPARAM) -> i16 {
    (((lparam.0 as usize) & 0xFFFF_0000) >> 16) as u16 as i16
}

/// Implementation of the `MAKELPARAM` macro.
/// Inverse of [GET_X_LPARAM] and [GET_Y_LPARAM] to put the (`x`, `y`) signed
/// coordinates/values back into an [LPARAM].
#[allow(non_snake_case)]
#[inline]
pub fn MAKELPARAM(x: i16, y: i16) -> LPARAM {
    LPARAM(((x as u16 as u32) | ((y as u16 as u32) << 16)) as usize as _)
}

fn main() -> Result<()> {
    // Example: Create an LPARAM with coordinates (100, 200)
    let x: i16 = 100;
    let y: i16 = 200;
    let lparam = MAKELPARAM(x, y);

    // Extract the coordinates back
    let extracted_x = GET_X_LPARAM(lparam);
    let extracted_y = GET_Y_LPARAM(lparam);

    println!("Original coordinates: ({}, {})", x, y);
    println!("Extracted coordinates: ({}, {})", extracted_x, extracted_y);

    // Verify they match
    assert_eq!(x, extracted_x);
    assert_eq!(y, extracted_y);

    // Example with negative coordinates
    let neg_x: i16 = -50;
    let neg_y: i16 = -75;
    let neg_lparam = MAKELPARAM(neg_x, neg_y);

    let extracted_neg_x = GET_X_LPARAM(neg_lparam);
    let extracted_neg_y = GET_Y_LPARAM(neg_lparam);

    println!("\nNegative coordinates: ({}, {})", neg_x, neg_y);
    println!(
        "Extracted negative coordinates: ({}, {})",
        extracted_neg_x, extracted_neg_y
    );

    assert_eq!(neg_x, extracted_neg_x);
    assert_eq!(neg_y, extracted_neg_y);

    println!("\nAll coordinate extractions successful!");

    Ok(())
}
