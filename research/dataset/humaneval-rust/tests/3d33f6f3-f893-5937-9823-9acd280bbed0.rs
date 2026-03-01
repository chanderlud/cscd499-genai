
#[cfg(test)]
mod tests {
    use super::*;

#[test]
    fn test_parse_nested_parens() {
        assert!(
            parse_nested_parens(String::from("(()()) ((())) () ((())()())")) == vec![2, 3, 1, 3]
        );
        assert!(parse_nested_parens(String::from("() (()) ((())) (((())))")) == vec![1, 2, 3, 4]);
        assert!(parse_nested_parens(String::from("(()(())((())))")) == vec![4]);
    }

}
