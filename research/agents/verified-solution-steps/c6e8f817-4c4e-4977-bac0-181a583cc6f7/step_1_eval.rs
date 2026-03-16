use windows::core::PWSTR;

pub fn normalize_separators(buf: PWSTR) -> usize {
    // Handle null pointer case
    if buf.is_null() {
        return 0;
    }

    let mut count = 0;
    let mut ptr = buf.0;

    // SAFETY: We've checked for null, and we're iterating until we find the null terminator.
    // The caller guarantees the pointer points to a valid null-terminated UTF-16 string.
    unsafe {
        while *ptr != 0 {
            if *ptr == b'\\' as u16 {
                *ptr = b'/' as u16;
                count += 1;
            }
            ptr = ptr.add(1);
        }
    }

    count
}