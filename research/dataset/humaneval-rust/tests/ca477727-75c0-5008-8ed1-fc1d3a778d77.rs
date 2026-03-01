
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare() {
        assert_eq!(
            compare(vec![1, 2, 3, 4, 5, 1], vec![1, 2, 3, 4, 2, -2]),
            vec![0, 0, 0, 0, 3, 3]
        );
        assert_eq!(
            compare(vec![0, 5, 0, 0, 0, 4], vec![4, 1, 1, 0, 0, -2]),
            vec![4, 4, 1, 0, 0, 6]
        );
        assert_eq!(
            compare(vec![1, 2, 3, 4, 5, 1], vec![1, 2, 3, 4, 2, -2]),
            vec![0, 0, 0, 0, 3, 3]
        );
        assert_eq!(
            compare(vec![0, 0, 0, 0, 0, 0], vec![0, 0, 0, 0, 0, 0]),
            vec![0, 0, 0, 0, 0, 0]
        );
        assert_eq!(compare(vec![1, 2, 3], vec![-1, -2, -3]), vec![2, 4, 6]);
        assert_eq!(
            compare(vec![1, 2, 3, 5], vec![-1, 2, 3, 4]),
            vec![2, 0, 0, 1]
        );
    }

}
