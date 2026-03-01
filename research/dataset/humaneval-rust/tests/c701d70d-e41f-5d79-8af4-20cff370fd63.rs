
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pairs_sum_to_zero() {
        assert!(pairs_sum_to_zero(vec![1, 3, 5, 0]) == false);
        assert!(pairs_sum_to_zero(vec![1, 3, -2, 1]) == false);
        assert!(pairs_sum_to_zero(vec![1, 2, 3, 7]) == false);
        assert!(pairs_sum_to_zero(vec![2, 4, -5, 3, 5, 7]) == true);
        assert!(pairs_sum_to_zero(vec![1]) == false);
        assert!(pairs_sum_to_zero(vec![-3, 9, -1, 3, 2, 30]) == true);
        assert!(pairs_sum_to_zero(vec![-3, 9, -1, 3, 2, 31]) == true);
        assert!(pairs_sum_to_zero(vec![-3, 9, -1, 4, 2, 30]) == false);
        assert!(pairs_sum_to_zero(vec![-3, 9, -1, 4, 2, 31]) == false);
    }

}
