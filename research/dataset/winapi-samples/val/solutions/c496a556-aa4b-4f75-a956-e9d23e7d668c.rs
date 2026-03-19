use windows::core::Result;

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

fn from_wide_ptr(ptr: *const u16) -> String {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    assert!(!ptr.is_null());
    let len = unsafe {
        (0..isize::MAX)
            .position(|i| *ptr.offset(i) == 0)
            .unwrap_or(0)
    };
    if len > 0 {
        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
        OsString::from_wide(slice).to_string_lossy().into_owned()
    } else {
        String::new()
    }
}

fn main() -> Result<()> {
    let test_string = "Hello, Windows!";
    println!("Original string: {}", test_string);

    // Convert to wide string
    let wide = wide_null(std::ffi::OsStr::new(test_string));
    println!("Wide string length (including null): {}", wide.len());

    // Convert back from wide string
    let converted = from_wide_ptr(wide.as_ptr());
    println!("Converted back: {}", converted);

    // Verify round-trip
    assert_eq!(test_string, converted);
    println!("Round-trip conversion successful!");

    Ok(())
}
