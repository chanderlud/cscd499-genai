// Get window frame thickness using GetSystemMetrics

use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXPADDEDBORDER, SM_CXSIZEFRAME,
};

fn get_frame_thickness() -> i32 {
    // SAFETY: GetSystemMetrics is a safe Win32 API call with no invalid parameters
    let (resize_frame_thickness, padding_thickness) = unsafe {
        (
            GetSystemMetrics(SM_CXSIZEFRAME),
            GetSystemMetrics(SM_CXPADDEDBORDER),
        )
    };
    resize_frame_thickness + padding_thickness
}

fn main() {
    let thickness = get_frame_thickness();
    println!("Window frame thickness: {} pixels", thickness);
}
