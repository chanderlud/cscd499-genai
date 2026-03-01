
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_words() {
        assert_eq!(split_words("Hello world!"), vec!["Hello", "world!"]);
        assert_eq!(split_words("Hello,world!"), vec!["Hello", "world!"]);
        assert_eq!(split_words("Hello world,!"), vec!["Hello", "world,!"]);
        assert_eq!(
            split_words("Hello,Hello,world !"),
            vec!["Hello,Hello,world", "!"]
        );
        assert_eq!(split_words("abcdef"), vec!["3"]);
        assert_eq!(split_words("aaabb"), vec!["2"]);
        assert_eq!(split_words("aaaBb"), vec!["1"]);
        assert_eq!(split_words(""), vec!["0"]);
    }

}
