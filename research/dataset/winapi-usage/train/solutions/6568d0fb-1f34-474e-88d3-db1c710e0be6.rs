use windows::core::{Error, Result, HRESULT};
use windows::Win32::Globalization::CompareStringOrdinal;

fn call_compare_string_ordinal() -> HRESULT {
    let s1: Vec<u16> = "hello".encode_utf16().chain(std::iter::once(0)).collect();
    let s2: Vec<u16> = "world".encode_utf16().chain(std::iter::once(0)).collect();

    // SAFETY: s1 and s2 are properly null-terminated UTF-16 slices.
    let result = unsafe { CompareStringOrdinal(&s1, &s2, false) };
    HRESULT(result.0)
}
