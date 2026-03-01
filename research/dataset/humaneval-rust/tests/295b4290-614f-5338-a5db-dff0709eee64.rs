
#[cfg(test)]
mod tests {
    use super::*;

 #[test]
    fn test_sum_product() {
        assert!(sum_product(vec![]) == (0, 1));
        assert!(sum_product(vec![1, 1, 1]) == (3, 1));
        assert!(sum_product(vec![100, 0]) == (100, 0));
        assert!(sum_product(vec![3, 5, 7]) == (3 + 5 + 7, 3 * 5 * 7));
        assert!(sum_product(vec![10]) == (10, 10));
    }

}
