use windows::core::HSTRING;

pub fn join_hstrings(parts: &[HSTRING]) -> HSTRING {
    let segments: Vec<&str> = parts
        .iter()
        .map(|h| h.to_string_lossy().trim_matches('\\'))
        .filter(|s| !s.is_empty())
        .collect();
    
    HSTRING::from(segments.join("\\"))
}