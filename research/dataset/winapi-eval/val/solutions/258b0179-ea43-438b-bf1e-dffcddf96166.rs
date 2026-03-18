use windows::core::HSTRING;

pub fn join_hstrings(parts: &[HSTRING]) -> HSTRING {
    let segments: Vec<String> = parts
        .iter()
        .map(|h| {
            let s = h.to_string_lossy();
            s.trim_matches('\\').to_string()
        })
        .filter(|s| !s.is_empty())
        .collect();

    HSTRING::from(segments.join("\\"))
}
