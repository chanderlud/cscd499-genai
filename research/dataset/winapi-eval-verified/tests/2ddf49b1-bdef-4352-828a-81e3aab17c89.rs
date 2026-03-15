#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn unique_id(tag: &str) -> String {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("{}_{}_{}", tag, std::process::id(), n)
    }

    #[test]
    fn test_reg_set_get_hkcu_basic() -> Result<()> {
        let path = format!(r"Software\MyApp\Tests\{}", unique_id("basic"));
        let name = "Answer";
        let value = "42";
        let result = reg_set_get_hkcu(&path, name, value)?;
        assert_eq!(result, value);
        Ok(())
    }

    #[test]
    fn test_reg_set_get_hkcu_empty_value() -> Result<()> {
        let path = format!(r"Software\MyApp\Tests\{}", unique_id("empty_value"));
        let name = "EmptyValue";
        let value = "";
        let result = reg_set_get_hkcu(&path, name, value)?;
        assert_eq!(result, value);
        Ok(())
    }

    #[test]
    fn test_reg_set_get_hkcu_long_value() -> Result<()> {
        let path = format!(r"Software\MyApp\Tests\{}", unique_id("long_value"));
        let name = "LongValue";
        let value = "a".repeat(1000);
        let result = reg_set_get_hkcu(&path, name, &value)?;
        assert_eq!(result, value);
        Ok(())
    }

    #[test]
    fn test_reg_set_get_hkcu_special_chars() -> Result<()> {
        let path = format!(r"Software\MyApp\Tests\{}", unique_id("special_chars"));
        let name = "SpecialChars";
        let value = "Hello\nWorld\t123!@#";
        let result = reg_set_get_hkcu(&path, name, value)?;
        assert_eq!(result, value);
        Ok(())
    }

    #[test]
    fn test_reg_set_get_hkcu_unicode() -> Result<()> {
        let path = format!(r"Software\MyApp\Tests\{}", unique_id("unicode"));
        let name = "Unicode";
        let value = "こんにちは世界";
        let result = reg_set_get_hkcu(&path, name, value)?;
        assert_eq!(result, value);
        Ok(())
    }

    #[test]
    fn test_reg_set_get_hkcu_multiple_keys_same_name() -> Result<()> {
        let base = unique_id("multiple_keys");
        let path1 = format!(r"Software\MyApp\Tests\{}\Key1", base);
        let path2 = format!(r"Software\MyApp\Tests\{}\Key2", base);
        let name = "SharedName";
        let value1 = "first";
        let value2 = "second";

        let result1 = reg_set_get_hkcu(&path1, name, value1)?;
        let result2 = reg_set_get_hkcu(&path2, name, value2)?;

        assert_eq!(result1, value1);
        assert_eq!(result2, value2);
        Ok(())
    }

    #[test]
    fn test_reg_set_get_hkcu_empty_path() -> Result<()> {
        let path = "";
        let name = unique_id("empty_path");
        let value = "test";
        let result = reg_set_get_hkcu(path, &name, value)?;
        assert_eq!(result, value);
        Ok(())
    }

    #[test]
    fn test_reg_set_get_hkcu_empty_name() -> Result<()> {
        let path = format!(r"Software\MyApp\Tests\{}", unique_id("empty_name"));
        let name = "";
        let value = "test";
        let result = reg_set_get_hkcu(&path, name, value)?;
        assert_eq!(result, value);
        Ok(())
    }

    #[test]
    fn test_reg_set_get_hkcu_overwrite_existing_value() -> Result<()> {
        let path = format!(r"Software\MyApp\Tests\{}", unique_id("overwrite"));
        let name = "Answer";

        let first = reg_set_get_hkcu(&path, name, "first")?;
        let second = reg_set_get_hkcu(&path, name, "second")?;

        assert_eq!(first, "first");
        assert_eq!(second, "second");
        Ok(())
    }
}
