use windows::core::{Result, Error};
use windows::core::PCSTR;

pub fn pcstr_len(ptr: PCSTR) -> usize {
    // A null pointer has length 0
    if ptr.is_null() {
        return 0;
    }
    
    let mut len = 0;
    
    // SAFETY: We've checked that ptr is not null, and we're reading bytes
    // until we find a null terminator. This is safe as long as the PCSTR
    // points to a valid null-terminated string, which is the contract.
    unsafe {
        while *ptr.0.add(len) != 0 {
            len += 1;
        }
    }
    
    len
}