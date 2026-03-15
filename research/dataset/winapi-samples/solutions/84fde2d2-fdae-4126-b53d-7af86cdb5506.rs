// Calculate window insets for DPI-aware non-client area

use std::sync::OnceLock;
use windows::{
    core::{PCSTR, PCWSTR},
    Win32::{
        Foundation::{FARPROC, RECT},
        System::LibraryLoader::{GetProcAddress, LoadLibraryW},
        UI::WindowsAndMessaging::{
            GetSystemMetrics, SM_CXPADDEDBORDER, SM_CXSIZEFRAME, SYSTEM_METRICS_INDEX,
            USER_DEFAULT_SCREEN_DPI,
        },
    },
};

fn encode_wide(string: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    use std::os::windows::prelude::OsStrExt;
    string
        .as_ref()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn get_function_impl(library: &str, function: &str) -> FARPROC {
    let library = encode_wide(library);
    assert_eq!(function.chars().last(), Some('\0'));

    // SAFETY: LoadLibraryW and GetProcAddress are called with valid pointers
    let module = unsafe { LoadLibraryW(PCWSTR::from_raw(library.as_ptr())) }.unwrap_or_default();
    if module.is_invalid() {
        return None;
    }

    unsafe { GetProcAddress(module, PCSTR::from_raw(function.as_ptr())) }
}

macro_rules! get_function {
    ($lib:expr, $func:ident) => {
        get_function_impl($lib, concat!(stringify!($func), '\0'))
            .map(|f| unsafe { std::mem::transmute::<_, $func>(f) })
    };
}

type GetSystemMetricsForDpi =
    unsafe extern "system" fn(nindex: SYSTEM_METRICS_INDEX, dpi: u32) -> i32;

static GET_SYSTEM_METRICS_FOR_DPI: OnceLock<Option<GetSystemMetricsForDpi>> = OnceLock::new();

/// Gets system metrics with DPI awareness, falling back to non-DPI-aware version
/// if the DPI-aware function is not available.
unsafe fn get_system_metrics_for_dpi(nindex: SYSTEM_METRICS_INDEX, dpi: u32) -> i32 {
    let get_system_metrics_for_dpi = GET_SYSTEM_METRICS_FOR_DPI
        .get_or_init(|| get_function!("user32.dll", GetSystemMetricsForDpi));

    if let Some(get_system_metrics_for_dpi) = get_system_metrics_for_dpi {
        get_system_metrics_for_dpi(nindex, dpi)
    } else {
        GetSystemMetrics(nindex)
    }
}

/// Gets the frame thickness for a given DPI.
fn get_frame_thickness(dpi: u32) -> i32 {
    // SAFETY: get_system_metrics_for_dpi is safe to call with valid parameters
    let resize_frame_thickness = unsafe { get_system_metrics_for_dpi(SM_CXSIZEFRAME, dpi) };
    let padding_thickness = unsafe { get_system_metrics_for_dpi(SM_CXPADDEDBORDER, dpi) };
    resize_frame_thickness + padding_thickness
}

/// Calculates window insets for DPI-aware non-client area rendering.
/// This is used in WM_NCCALCSIZE to adjust the client area.
fn calculate_insets_for_dpi(dpi: u32) -> RECT {
    let frame_thickness = get_frame_thickness(dpi);

    // On Windows 11 (build 22000+), top inset needs adjustment
    let top_inset = if cfg!(target_os = "windows") {
        // Simplified version check - in real code you'd check the actual Windows version
        let is_windows_11 = true; // Assume Windows 11 for this example
        if is_windows_11 {
            (dpi as f32 / USER_DEFAULT_SCREEN_DPI as f32).round() as i32
        } else {
            0
        }
    } else {
        0
    };

    RECT {
        left: frame_thickness,
        top: top_inset,
        right: frame_thickness,
        bottom: frame_thickness,
    }
}

fn main() {
    // Example: Calculate insets for 96 DPI (100% scaling)
    let dpi = USER_DEFAULT_SCREEN_DPI;
    let insets = calculate_insets_for_dpi(dpi);

    println!("Window insets at {} DPI:", dpi);
    println!("  Left: {}", insets.left);
    println!("  Top: {}", insets.top);
    println!("  Right: {}", insets.right);
    println!("  Bottom: {}", insets.bottom);

    // Example: Calculate insets for 144 DPI (150% scaling)
    let dpi = 144;
    let insets = calculate_insets_for_dpi(dpi);

    println!("\nWindow insets at {} DPI:", dpi);
    println!("  Left: {}", insets.left);
    println!("  Top: {}", insets.top);
    println!("  Right: {}", insets.right);
    println!("  Bottom: {}", insets.bottom);
}
