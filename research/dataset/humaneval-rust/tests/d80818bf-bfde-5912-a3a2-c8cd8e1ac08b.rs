
#[cfg(test)]
mod tests {
    use super::*;

 #[test]
    fn test_triples_sum_to_zero() {
        assert!(triples_sum_to_zero(vec![1, 3, 5, 0]) == false);
        assert!(triples_sum_to_zero(vec![1, 3, 5, -1]) == false);
        assert!(triples_sum_to_zero(vec![1, 3, -2, 1]) == true);
        assert!(triples_sum_to_zero(vec![1, 2, 3, 7]) == false);
        assert!(triples_sum_to_zero(vec![1, 2, 5, 7]) == false);
        assert!(triples_sum_to_zero(vec![2, 4, -5, 3, 9, 7]) == true);
        assert!(triples_sum_to_zero(vec![1]) == false);
        assert!(triples_sum_to_zero(vec![1, 3, 5, -100]) == false);
        assert!(triples_sum_to_zero(vec![100, 3, 5, -100]) == false);
    }

}
