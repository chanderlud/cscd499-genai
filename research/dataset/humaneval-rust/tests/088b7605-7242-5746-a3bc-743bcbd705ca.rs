
#[cfg(test)]
mod tests {
    use super::*;

#[test]
    fn test_separate_paren_groups() {
        assert_eq!(
            separate_paren_groups(String::from("(()()) ((())) () ((())()())")),
            vec!["(()())", "((()))", "()", "((())()())"]
        );
        assert_eq!(
            separate_paren_groups(String::from("() (()) ((())) (((())))")),
            vec!["()", "(())", "((()))", "(((())))"]
        );
        assert_eq!(
            separate_paren_groups(String::from("(()(())((())))")),
            vec!["(()(())((())))"]
        );
        assert_eq!(
            separate_paren_groups(String::from("( ) (( )) (( )( ))")),
            vec!["()", "(())", "(()())"]
        );
    }

}
