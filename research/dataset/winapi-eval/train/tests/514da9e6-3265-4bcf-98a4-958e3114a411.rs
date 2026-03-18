#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_user_sid_string_returns_ok() {
        let result = current_user_sid_string();
        assert!(result.is_ok(), "Expected Ok result, got {:?}", result);
    }

    #[test]
    fn test_current_user_sid_string_returns_valid_sid_format() {
        let sid = current_user_sid_string().unwrap();
        assert!(sid.starts_with("S-"), "SID should start with 'S-'");
        assert!(sid.contains('-'), "SID should contain '-' separator");
        assert!(!sid.is_empty(), "SID should not be empty");
    }

    #[test]
    fn test_current_user_sid_string_is_deterministic() {
        let sid1 = current_user_sid_string().unwrap();
        let sid2 = current_user_sid_string().unwrap();
        assert_eq!(
            sid1, sid2,
            "SID should be deterministic across multiple calls"
        );
    }

    #[test]
    fn test_current_user_sid_string_has_expected_length() {
        let sid = current_user_sid_string().unwrap();
        // SIDs typically have a reasonable length (e.g., S-1-5-21-... is usually 20-50 chars)
        assert!(sid.len() >= 10, "SID should have reasonable length");
        assert!(sid.len() <= 100, "SID should not be excessively long");
    }

    #[test]
    fn test_current_user_sid_string_contains_valid_sid_components() {
        let sid = current_user_sid_string().unwrap();
        // SIDs have the format S-R-I-S... where:
        // S: literal "S"
        // R: revision (always 1)
        // I: identifier authority (48-bit value, typically 5 for NT Authority)
        // S: subauthorities (variable number of 32-bit values)
        let parts: Vec<&str> = sid.split('-').collect();
        assert!(
            parts.len() >= 3,
            "SID should have at least 3 components (S-R-I)"
        );

        // First part should be "S"
        assert_eq!(parts[0], "S", "First component should be 'S'");

        // Second part should be revision, always "1"
        assert_eq!(parts[1], "1", "Second component should be revision '1'");

        // Third part should be identifier authority (numeric)
        assert!(
            parts[2].parse::<u64>().is_ok(),
            "Third component should be numeric identifier authority"
        );

        // Remaining parts should be numeric subauthorities
        for part in &parts[3..] {
            assert!(
                part.parse::<u32>().is_ok(),
                "Subauthority '{}' should be numeric",
                part
            );
        }
    }
}
