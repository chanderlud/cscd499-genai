use windows::core::PCWSTR;

pub fn pcwstr_len(ptr: PCWSTR) -> usize {
    // Check for null pointer first
    if ptr.0.is_null() {
        return 0;
    }

    let mut len = 0;
    // SAFETY: We've checked for null above. The caller must ensure the pointer
    // points to a valid null-terminated UTF-16 string. We only read until we
    // find the null terminator, which is the contract of PCWSTR.
    unsafe {
        let mut current = ptr.0;
        while *current != 0 {
            len += 1;
            current = current.add(1);
        }
    }
    len
}