#[cfg(test)]
mod tests {
    use windows::core::HSTRING;
    use super::*;

    #[test]
    fn joins_and_trims_segments() {
        let parts = [
            HSTRING::from("C:\\"),
            HSTRING::from("\\tmp\\"),
            HSTRING::from("a"),
            HSTRING::from("b\\"),
        ];
        assert_eq!(join_hstrings(&parts).to_string_lossy(), r"C:\tmp\a\b");
    }

    #[test]
    fn skips_empty_segments() {
        let parts = [
            HSTRING::from(""),
            HSTRING::from("\\"),
            HSTRING::from("logs"),
            HSTRING::from(""),
        ];
        assert_eq!(join_hstrings(&parts).to_string_lossy(), "logs");
    }

    #[test]
    fn all_empty_yields_empty_hstring() {
        let parts = [HSTRING::from(""), HSTRING::from("\\")];
        assert_eq!(join_hstrings(&parts).to_string_lossy(), "");
    }
}
