// Auto-generated tests for: 97d06dec-c0ff-4354-b131-25e7dd1dfa0a.md
// Model: arcee-ai/trinity-large-preview:free
// Extraction: rust

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
        let items = vec!["\0".to_string(), "\n".to_string(), "\t".to_string()];
        let out = reg_multisz_roundtrip("Software\\RustWinApiProblems\\Special", "Items", &items)?;
        assert_eq!(out, items);
        Ok(())
    }

    #[test]
    fn test_reg_multisz_roundtrip_same_subkey_different_values() -> io::Result<()> {
        let items1 = vec!["first".to_string()];
        let items2 = vec!["second".to_string()];
        let out1 = reg_multisz_roundtrip("Software\\RustWinApiProblems\\SameKey", "First", &items1)?;
        let out2 = reg_multisz_roundtrip("Software\\RustWinApiProblems\\SameKey", "Second", &items2)?;
        assert_eq!(out1, items1);
        assert_eq!(out2, items2);
        Ok(())
    }

    #[test]
    fn test_reg_multisz_roundtrip_nested_subkeys() -> io::Result<()> {
        let items = vec!["nested".to_string()];
        let out = reg_multisz_roundtrip("Software\\RustWinApiProblems\\Nested\\Level1\\Level2", "Items", &items)?;
        assert_eq!(out, items);
        Ok(())
    }
}
