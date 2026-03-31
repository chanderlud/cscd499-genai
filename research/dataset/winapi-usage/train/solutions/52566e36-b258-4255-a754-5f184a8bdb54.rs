use windows::Win32::System::Com::CLSIDFromProgID;

fn call_clsid_from_prog_id() -> windows::core::HRESULT {
    // SAFETY: CLSIDFromProgID expects a valid null-terminated wide string.
    // The `w!` macro safely creates a PCWSTR at compile time.
    unsafe {
        CLSIDFromProgID(windows::core::w!("Excel.Application"))
            .map(|_| windows::core::HRESULT(0))
            .unwrap_or_else(|e| e.code())
    }
}
