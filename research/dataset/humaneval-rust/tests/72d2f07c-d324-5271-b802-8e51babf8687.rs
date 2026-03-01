
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derivative() {
        assert!(derivative(vec![3, 1, 2, 4, 5]) == vec![1, 4, 12, 20]);
        assert!(derivative(vec![1, 2, 3]) == vec![2, 6]);
        assert!(derivative(vec![3, 2, 1]) == vec![2, 2]);
        assert!(derivative(vec![3, 2, 1, 0, 4]) == vec![2, 2, 0, 16]);
        assert!(derivative(vec![1]) == vec![]);
    }

}
