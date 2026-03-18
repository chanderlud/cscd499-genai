#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;
    use windows::{
        core::{HSTRING, Result},
        Foundation::PropertyValue,
        Storage::ApplicationData,
    };

    fn cleanup_key(key: &str) -> Result<()> {
        let values = ApplicationData::Current()?.LocalSettings()?.Values()?;
        let key = HSTRING::from(key);

        if values.HasKey(&key)? {
            values.Remove(&key)?;
        }

        Ok(())
    }

    fn insert_raw_int32(key: &str, value: i32) -> Result<()> {
        let values = ApplicationData::Current()?.LocalSettings()?.Values()?;
        let key = HSTRING::from(key);
        let value = PropertyValue::CreateInt32(value)?;
        values.Insert(&key, &value)?;
        Ok(())
    }

    #[test]
    fn local_settings_round_trip() -> Result<()> {
        let key = "unit_test.local_settings_round_trip";

        cleanup_key(key)?;
        local_settings_set(key, "dark")?;

        assert_eq!(local_settings_get(key)?, Some("dark".to_string()));

        cleanup_key(key)?;
        Ok(())
    }

    #[test]
    fn local_settings_overwrites_existing_value() -> Result<()> {
        let key = "unit_test.local_settings_overwrites_existing_value";

        cleanup_key(key)?;
        local_settings_set(key, "light")?;
        local_settings_set(key, "dark")?;

        assert_eq!(local_settings_get(key)?, Some("dark".to_string()));

        cleanup_key(key)?;
        Ok(())
    }

    #[test]
    fn local_settings_missing_key_returns_none() -> Result<()> {
        let key = "unit_test.local_settings_missing_key_returns_none";

        cleanup_key(key)?;
        assert_eq!(local_settings_get(key)?, None);

        Ok(())
    }

    #[test]
    fn local_settings_values_are_isolated_by_key() -> Result<()> {
        let key_a = "unit_test.local_settings_values_are_isolated_by_key.a";
        let key_b = "unit_test.local_settings_values_are_isolated_by_key.b";

        cleanup_key(key_a)?;
        cleanup_key(key_b)?;

        local_settings_set(key_a, "alpha")?;
        local_settings_set(key_b, "beta")?;

        assert_eq!(local_settings_get(key_a)?, Some("alpha".to_string()));
        assert_eq!(local_settings_get(key_b)?, Some("beta".to_string()));

        cleanup_key(key_a)?;
        cleanup_key(key_b)?;
        Ok(())
    }

    #[test]
    fn local_settings_round_trip_empty_string() -> Result<()> {
        let key = "unit_test.local_settings_round_trip_empty_string";

        cleanup_key(key)?;
        local_settings_set(key, "")?;

        assert_eq!(local_settings_get(key)?, Some(String::new()));

        cleanup_key(key)?;
        Ok(())
    }

    #[test]
    fn local_settings_round_trip_unicode_string() -> Result<()> {
        let key = "unit_test.local_settings_round_trip_unicode_string";

        cleanup_key(key)?;
        local_settings_set(key, "héllo 🌍 こんにちは")?;

        assert_eq!(
            local_settings_get(key)?,
            Some("héllo 🌍 こんにちは".to_string())
        );

        cleanup_key(key)?;
        Ok(())
    }

    #[test]
    fn local_settings_existing_non_string_value_returns_error() -> Result<()> {
        let key = "unit_test.local_settings_existing_non_string_value_returns_error";

        cleanup_key(key)?;
        insert_raw_int32(key, 42)?;

        assert!(local_settings_get(key).is_err());

        cleanup_key(key)?;
        Ok(())
    }
}
