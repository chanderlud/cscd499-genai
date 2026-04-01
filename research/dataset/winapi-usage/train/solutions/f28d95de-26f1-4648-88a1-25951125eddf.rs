use windows::core::{Error, Result};
use windows::Win32::UI::Input::Ime::{ImmGetCompositionStringW, HIMC, IME_COMPOSITION_STRING};

fn call_imm_get_composition_string_w() -> windows::core::Result<i32> {
    // Create a buffer for the composition string (256 wide characters)
    let mut buffer: Vec<u16> = vec![0; 256];

    // Create a dummy HIMC (in real code, this would come from ImmGetContext)
    let himc = HIMC(std::ptr::null_mut());

    // Use GCS_COMPSTR (0) to get the composition string
    let gcs = IME_COMPOSITION_STRING(0);

    // Call the API
    let result = unsafe {
        ImmGetCompositionStringW(
            himc,
            gcs,
            Some(buffer.as_mut_ptr() as *mut core::ffi::c_void),
            (buffer.len() * 2) as u32, // byte length for wide characters
        )
    };

    Ok(result)
}
