
#[cfg(test)]
mod tests {
    use super::*;

 #[test]
    fn test_intersperse() {
        assert!(intersperse(vec![], 7) == vec![]);
        assert!(intersperse(vec![5, 6, 3, 2], 8) == vec![5, 8, 6, 8, 3, 8, 2]);
        assert!(intersperse(vec![2, 2, 2], 2) == vec![2, 2, 2, 2, 2]);
    }

}
