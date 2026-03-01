
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_median() {
        assert!(median(vec![3, 1, 2, 4, 5]) == 3.0);
        assert!(median(vec![-10, 4, 6, 1000, 10, 20]) == 8.0);
        assert!(median(vec![5]) == 5.0);
        assert!(median(vec![6, 5]) == 5.5);
        assert!(median(vec![8, 1, 3, 9, 9, 2, 7]) == 7.0);
    }

}
