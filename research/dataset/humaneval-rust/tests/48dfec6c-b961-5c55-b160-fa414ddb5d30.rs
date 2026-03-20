#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string_is_palindrome() {
        assert!(is_palindrome_10(""));
    }

    #[test]
    fn single_character_is_palindrome() {
        assert!(is_palindrome_10("a"));
    }

    #[test]
    fn two_same_characters_is_palindrome() {
        assert!(is_palindrome_10("aa"));
    }

    #[test]
    fn two_different_characters_is_not_palindrome() {
        assert!(!is_palindrome_10("ab"));
    }

    #[test]
    fn odd_length_palindrome() {
        assert!(is_palindrome_10("racecar"));
    }

    #[test]
    fn even_length_palindrome() {
        assert!(is_palindrome_10("abba"));
    }

    #[test]
    fn simple_non_palindrome() {
        assert!(!is_palindrome_10("hello"));
    }

    #[test]
    fn near_palindrome_is_not_palindrome() {
        assert!(!is_palindrome_10("abca"));
    }

    #[test]
    fn repeated_characters_palindrome() {
        assert!(is_palindrome_10("aaaaaa"));
    }

    #[test]
    fn longer_palindrome() {
        assert!(is_palindrome_10("abcdedcba"));
    }

    #[test]
    fn numeric_string_palindrome() {
        assert!(is_palindrome_10("12321"));
    }

    #[test]
    fn numeric_string_non_palindrome() {
        assert!(!is_palindrome_10("12345"));
    }

    #[test]
    fn case_sensitive_check() {
        assert!(!is_palindrome_10("Aa"));
    }

    #[test]
    fn punctuation_is_checked_literally() {
        assert!(is_palindrome_10("a!a"));
    }

    #[test]
    fn spaces_are_checked_literally() {
        assert!(!is_palindrome_10("nurses run"));
    }
}