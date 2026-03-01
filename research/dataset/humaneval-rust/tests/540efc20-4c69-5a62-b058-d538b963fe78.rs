
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_max() {
        assert_eq!(find_max(vec!["name", "of", "string"]), "string");
        assert_eq!(find_max(vec!["name", "enam", "game"]), "enam");
        assert_eq!(find_max(vec!["aaaaaaa", "bb", "cc"]), "aaaaaaa");
        assert_eq!(find_max(vec!["abc", "cba"]), "abc");
        assert_eq!(
            find_max(vec!["play", "this", "game", "of", "footbott"]),
            "footbott"
        );
        assert_eq!(find_max(vec!["we", "are", "gonna", "rock"]), "gonna");
        assert_eq!(find_max(vec!["we", "are", "a", "mad", "nation"]), "nation");
        assert_eq!(find_max(vec!["this", "is", "a", "prrk"]), "this");
        assert_eq!(find_max(vec!["b"]), "b");
        assert_eq!(find_max(vec!["play", "play", "play"]), "play");
    }

}
