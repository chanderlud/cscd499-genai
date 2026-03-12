#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reg_set_get_hkcu_basic() -> Result<()> {
        let path = r"Software\MyApp\Tests";
        let name = "Answer";
        let value = "42";
        let result = reg_set_get_hkcu(path, name, value)?;
        assert_eq!(result, value);
        Ok(())
    }

    #[test]
    fn test_reg_set_get_hkcu_empty_value() -> Result<()> {
        let path = r"Software\MyApp\Tests";
        let name = "EmptyValue";
        let value = "";
        let result = reg_set_get_hkcu(path, name, value)?;
        assert_eq!(result, value);
        Ok(())
    }

    #[test]
    fn test_reg_set_get_hkcu_long_value() -> Result<()> {
        let path = r"Software\MyApp\Tests";
        let name = "LongValue";
        let value = "a".repeat(1000);
        let result = reg_set_get_hkcu(path, name, &value)?;
        assert_eq!(result, value);
        Ok(())
    }

    #[test]
    fn test_reg_set_get_hkcu_special_chars() -> Result<()> {
        let path = r"Software\MyApp\Tests";
        let name = "SpecialChars";
        let value = "Hello\nWorld\t123!@#";
        let result = reg_set_get_hkcu(path, name, value)?;
        assert_eq!(result, value);
        Ok(())
    }

    #[test]
    fn test_reg_set_get_hkcu_unicode() -> Result<()> {
        let path = r"Software\MyApp\Tests";
        let name = "Unicode";
        let value = "こんにちは世界";
        let result = reg_set_get_hkcu(path, name, value)?;
        assert_eq!(result, value);
        Ok(())
    }

    #[test]
    fn test_reg_set_get_hkcu_multiple_keys_same_name() -> Result<()> {
        let path1 = r"Software\MyApp\Tests\Key1";
        let path2 = r"Software\MyApp\Tests\Key2";
        let name = "SharedName";
        let value1 = "first";
        let value2 = "second";

        let result1 = reg_set_get_hkcu(path1, name, value1)?;
        let result2 = reg_set_get_hkcu(path2, name, value2)?;

        assert_eq!(result1, value1);
        assert_eq!(result2, value2);
        Ok(())
    }

    #[test]
    fn test_reg_set_get_hkcu_empty_path() -> Result<()> {
        let path = "";
        let name = "EmptyPath";
        let value = "test";
        let result = reg_set_get_hkcu(path, name, value)?;
        assert_eq!(result, value);
        Ok(())
    }

    #[test]
    fn test_reg_set_get_hkcu_empty_name() -> Result<()> {
        let path = r"Software\MyApp\Tests";
        let name = "";
        let value = "test";
        let result = reg_set_get_hkcu(path, name, value)?;
        assert_eq!(result, value);
        Ok(())
    }
}
