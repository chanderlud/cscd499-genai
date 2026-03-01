
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_equal_to_sum_even() {
        assert_eq!(is_equal_to_sum_even(4), false);
        assert_eq!(is_equal_to_sum_even(6), false);
        assert_eq!(is_equal_to_sum_even(8), true);
        assert_eq!(is_equal_to_sum_even(10), true);
        assert_eq!(is_equal_to_sum_even(11), false);
        assert_eq!(is_equal_to_sum_even(12), true);
        assert_eq!(is_equal_to_sum_even(13), false);
        assert_eq!(is_equal_to_sum_even(16), true);
    }

}
