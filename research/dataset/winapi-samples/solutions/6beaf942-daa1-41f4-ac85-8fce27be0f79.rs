// Get system metrics with DPI awareness and fallback

use std::sync::OnceLock;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::FARPROC,
        System::LibraryLoader::{GetProcAddress, LoadLibraryW},
        UI::WindowsAndMessaging::{
            GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN, SYSTEM_METRICS_INDEX,
        },
    },
};

fn encode_wide(string: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    string.as_ref().encode_wide().chain(once(0)).collect()
}

unsafe fn get_function_impl(library: &str, function: &str) -> FARPROC {
    let library = encode_wide(library);
    assert_eq!(function.chars().last(), Some('\0'));

    let module = LoadLibraryW(PCWSTR::from_raw(library.as_ptr())).unwrap_or_default();
    if module.is_invalid() {
        return None;
    }

    GetProcAddress(module, windows::core::PCSTR::from_raw(function.as_ptr()))
}

type GetSystemMetricsForDpi =
    unsafe extern "system" fn(nindex: SYSTEM_METRICS_INDEX, dpi: u32) -> i32;

static GET_SYSTEM_METRICS_FOR_DPI: OnceLock<Option<GetSystemMetricsForDpi>> = OnceLock::new();

fn get_system_metrics_for_dpi(nindex: SYSTEM_METRICS_INDEX, dpi: u32) -> i32 {
    if let Some(func) = GET_SYSTEM_METRICS_FOR_DPI.get_or_init(|| {
        // SAFETY: Loading a function from a system DLL
        let f = unsafe { get_function_impl("user32.dll", "GetSystemMetricsForDpi\0") };
        f.map(|f| unsafe { std::mem::transmute(f) })
    }) {
        // SAFETY: Calling a valid function pointer with correct parameters
        unsafe { func(nindex, dpi) }
    } else {
        // SAFETY: GetSystemMetrics is a valid Win32 API call
        unsafe { GetSystemMetrics(nindex) }
    }
}

fn main() -> windows::core::Result<()> {
    // Use 96 DPI (standard) for the example
    let dpi = 96;

    let width = get_system_metrics_for_dpi(SM_CXSCREEN, dpi);
    let height = get_system_metrics_for_dpi(SM_CYSCREEN, dpi);

    println!("Screen dimensions at {} DPI: {}x{}", dpi, width, height);

    Ok(())
}
