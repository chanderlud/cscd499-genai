
#[cfg(test)]
mod tests {
    use super::*;

 #[test]
    fn test_all_prefixes() {
        let v_empty: Vec<String> = vec![];
        assert!(all_prefixes(String::from("")) == v_empty);
        assert!(
            all_prefixes(String::from("asdfgh"))
                == vec!["a", "as", "asd", "asdf", "asdfg", "asdfgh"]
        );
        assert!(all_prefixes(String::from("WWW")) == vec!["W", "WW", "WWW"]);
    }

}
