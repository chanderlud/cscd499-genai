#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_reg_multisz_roundtrip_basic() -> io::Result<()> {
        let items = vec!["one".to_string(), "two".to_string()];
        let out = reg_multisz_roundtrip("Software\\RustWinApiProblems\\Basic", "Items", &items)?;
        assert_eq!(out, items);
        Ok(())
    }

    #[test]
    fn test_reg_multisz_roundtrip_empty_items() -> io::Result<()> {
        let items: Vec<String> = vec![];
        let out = reg_multisz_roundtrip("Software\\RustWinApiProblems\\Empty", "Items", &items)?;
        assert_eq!(out, items);
        Ok(())
    }

    #[test]
    fn test_reg_multisz_roundtrip_single_item() -> io::Result<()> {
        let items = vec!["only".to_string()];
        let out = reg_multisz_roundtrip("Software\\RustWinApiProblems\\Single", "Items", &items)?;
        assert_eq!(out, items);
        Ok(())
    }

    #[test]
    fn test_reg_multisz_roundtrip_unicode() -> io::Result<()> {
        let items = vec!["café".to_string(), "日本語".to_string(), "😊".to_string()];
        let out = reg_multisz_roundtrip("Software\\RustWinApiProblems\\Unicode", "Items", &items)?;
        assert_eq!(out, items);
        Ok(())
    }

    #[test]
    fn test_reg_multisz_roundtrip_long_strings() -> io::Result<()> {
        let long = "a".repeat(1000);
        let items = vec![long.clone(), long.clone()];
        let out = reg_multisz_roundtrip("Software\\RustWinApiProblems\\Long", "Items", &items)?;
        assert_eq!(out, items);
        Ok(())
    }

    #[test]
    fn test_reg_multisz_roundtrip_special_chars() -> io::Result<()> {
        // Note: REG_MULTI_SZ cannot store embedded NUL characters; they would terminate the string early.
        // This test uses other special characters that are valid in REG_MULTI_SZ.
        let items = vec!["\n".to_string(), "\t".to_string()];
        let out = reg_multisz_roundtrip("Software\\RustWinApiProblems\\Special", "Items", &items)?;
        assert_eq!(out, items);
        Ok(())
    }

    #[test]
    fn test_reg_multisz_roundtrip_same_subkey_different_values() -> io::Result<()> {
        let items1 = vec!["first".to_string()];
        let items2 = vec!["second".to_string()];
        let out1 =
            reg_multisz_roundtrip("Software\\RustWinApiProblems\\SameKey", "First", &items1)?;
        let out2 =
            reg_multisz_roundtrip("Software\\RustWinApiProblems\\SameKey", "Second", &items2)?;
        assert_eq!(out1, items1);
        assert_eq!(out2, items2);
        Ok(())
    }

    #[test]
    fn test_reg_multisz_roundtrip_nested_subkeys() -> io::Result<()> {
        let items = vec!["nested".to_string()];
        let out = reg_multisz_roundtrip(
            "Software\\RustWinApiProblems\\Nested\\Level1\\Level2",
            "Items",
            &items,
        )?;
        assert_eq!(out, items);
        Ok(())
    }

    #[test]
    fn test_reg_multisz_roundtrip_mixed_case() -> io::Result<()> {
        let items = vec![
            "MixedCase".to_string(),
            "UPPERcase".to_string(),
            "lowercase".to_string(),
        ];
        let out =
            reg_multisz_roundtrip("Software\\RustWinApiProblems\\MixedCase", "Items", &items)?;
        assert_eq!(out, items);
        Ok(())
    }

    #[test]
    fn test_reg_multisz_roundtrip_whitespace() -> io::Result<()> {
        let items = vec!["  spaced  ".to_string(), "  ".to_string()];
        let out =
            reg_multisz_roundtrip("Software\\RustWinApiProblems\\Whitespace", "Items", &items)?;
        assert_eq!(out, items);
        Ok(())
    }

    #[test]
    fn test_reg_multisz_roundtrip_rejects_empty_string_item() {
        let items = vec!["  spaced  ".to_string(), "  ".to_string(), "".to_string()];
        let err = reg_multisz_roundtrip(
            "Software\\RustWinApiProblems\\WhitespaceRejectEmpty",
            "Items",
            &items,
        )
            .unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn test_reg_multisz_roundtrip_numeric_strings() -> io::Result<()> {
        let items = vec!["123".to_string(), "456".to_string(), "789".to_string()];
        let out = reg_multisz_roundtrip("Software\\RustWinApiProblems\\Numeric", "Items", &items)?;
        assert_eq!(out, items);
        Ok(())
    }

    #[test]
    fn test_reg_multisz_roundtrip_control_chars() -> io::Result<()> {
        let items = vec!["\x01".to_string(), "\x02".to_string(), "\x1F".to_string()];
        let out = reg_multisz_roundtrip("Software\\RustWinApiProblems\\Control", "Items", &items)?;
        assert_eq!(out, items);
        Ok(())
    }
}
