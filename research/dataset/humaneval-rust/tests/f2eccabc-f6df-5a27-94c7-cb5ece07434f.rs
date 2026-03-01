
#[cfg(test)]
mod tests {
    use super::*;

#[test]
    fn test_longest() {
        assert!(longest(vec![]) == None);
        assert!(
            longest(vec!["x".to_string(), "y".to_string(), "z".to_string()])
                == Some("x".to_string())
        );
        assert!(
            longest(vec![
                "x".to_string(),
                "yyy".to_string(),
                "zzzz".to_string(),
                "www".to_string(),
                "kkkk".to_string(),
                "abc".to_string()
            ]) == Some("zzzz".to_string())
        );
    }

}
