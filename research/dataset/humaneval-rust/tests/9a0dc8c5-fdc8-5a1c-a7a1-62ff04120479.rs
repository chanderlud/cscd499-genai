
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_words_in_sentence() {
        assert_eq!(words_in_sentence("This is a test"), "is");
        assert_eq!(words_in_sentence("lets go for swimming"), "go for");
        assert_eq!(
            words_in_sentence("there is no place available here"),
            "there is no place"
        );
        assert_eq!(words_in_sentence("Hi I am Hussein"), "Hi am Hussein");
        assert_eq!(words_in_sentence("go for it"), "go for it");
        assert_eq!(words_in_sentence("here"), "");
        assert_eq!(words_in_sentence("here is"), "is");
    }

}
