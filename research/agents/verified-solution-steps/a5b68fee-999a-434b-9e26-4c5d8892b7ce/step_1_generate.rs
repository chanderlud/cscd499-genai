use windows::core::PCWSTR;

pub fn pcwstr_eq(ptr: PCWSTR, expected: &str) -> bool {
    // Handle null pointer case - only equal to empty string
    if ptr.is_null() {
        return expected.is_empty();
    }

    // Convert expected string to UTF-16 for comparison
    let expected_utf16: Vec<u16> = expected.encode_utf16().collect();
    
    // Safety: We've checked ptr is not null, and we'll only read until null terminator
    unsafe {
        let mut i = 0;
        loop {
            let current_char = *ptr.0.add(i);
            
            // Check if we've reached end of expected string
            if i == expected_utf16.len() {
                // PCWSTR must end here too (null terminator)
                return current_char == 0;
            }
            
            // Check if characters match
            if current_char != expected_utf16[i] {
                return false;
            }
            
            // Check if we've hit null terminator in PCWSTR
            if current_char == 0 {
                // PCWSTR ended before expected string
                return false;
            }
            
            i += 1;
        }
    }
}