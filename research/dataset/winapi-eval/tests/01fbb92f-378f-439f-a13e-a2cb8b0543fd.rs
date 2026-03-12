#[cfg(test)]
mod tests {
    use super::*;
    use windows::{
        core::{HSTRING, Result},
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
}