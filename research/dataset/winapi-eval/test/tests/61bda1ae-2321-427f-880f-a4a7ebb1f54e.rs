#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_resolve_export_manual_kernel32_getcurrentprocessid() {
        let path = Path::new(r"C:\Windows\System32\kernel32.dll");
        let symbol = "GetCurrentProcessId";
        let addr = resolve_export_manual(path, symbol).unwrap();
        assert!(
            addr != 0,
            "Expected non-zero address for kernel32!GetCurrentProcessId"
        );
    }

    #[test]
    fn test_resolve_export_manual_kernelbase_getcurrentprocessid() {
        let path = Path::new(r"C:\Windows\System32\kernelbase.dll");
        let symbol = "GetCurrentProcessId";
        let addr = resolve_export_manual(path, symbol).unwrap();
        assert!(
            addr != 0,
            "Expected non-zero address for kernelbase!GetCurrentProcessId"
        );
    }

    #[test]
    fn test_resolve_export_manual_forwarder_resolution() {
        let path = Path::new(r"C:\Windows\System32\kernel32.dll");
        let symbol = "GetSystemTimePreciseAsFileTime";
        let addr = resolve_export_manual(path, symbol).unwrap();
        assert!(addr != 0, "Expected non-zero address for kernel32!GetSystemTimePreciseAsFileTime (forwarder to kernelbase)");
    }

    #[test]
    fn test_resolve_export_manual_known_kernel32_export() {
        let path = Path::new(r"C:\Windows\System32\kernel32.dll");
        let symbol = "GetModuleHandleW";
        let addr = resolve_export_manual(path, symbol).unwrap();
        assert!(
            addr != 0,
            "Expected non-zero address for kernel32!GetModuleHandleW"
        );
    }

    #[test]
    fn test_resolve_export_manual_nonexistent_symbol() {
        let path = Path::new(r"C:\Windows\System32\kernel32.dll");
        let symbol = "NonExistentSymbol_123";
        let result = resolve_export_manual(path, symbol);
        assert!(result.is_err(), "Expected error for non-existent symbol");
    }

    #[test]
    fn test_resolve_export_manual_invalid_dll_path() {
        let path = Path::new(r"C:\Invalid\Path\NonExistent.dll");
        let symbol = "AnySymbol";
        let result = resolve_export_manual(path, symbol);
        assert!(result.is_err(), "Expected error for invalid DLL path");
    }

    #[test]
    fn test_resolve_export_manual_empty_symbol() {
        let path = Path::new(r"C:\Windows\System32\kernel32.dll");
        let symbol = "";
        let result = resolve_export_manual(path, symbol);
        assert!(result.is_err(), "Expected error for empty symbol name");
    }

    #[test]
    fn test_resolve_export_manual_system32_advapi32_regopenkeyexw() {
        let path = Path::new(r"C:\Windows\System32\advapi32.dll");
        let symbol = "RegOpenKeyExW";
        let addr = resolve_export_manual(path, symbol).unwrap();
        assert!(
            addr != 0,
            "Expected non-zero address for advapi32!RegOpenKeyExW"
        );
    }

    #[test]
    fn test_resolve_export_manual_kernel32_getlasterror() {
        let path = Path::new(r"C:\Windows\System32\kernel32.dll");
        let symbol = "GetLastError";
        let addr = resolve_export_manual(path, symbol).unwrap();
        assert!(
            addr != 0,
            "Expected non-zero address for kernel32!GetLastError"
        );
    }
}
