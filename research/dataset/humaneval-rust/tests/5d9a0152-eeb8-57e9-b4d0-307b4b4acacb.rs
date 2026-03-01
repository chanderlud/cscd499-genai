
#[cfg(test)]
mod tests {
    use super::*;

  #[test]
    fn test_sort_array() {
        assert!(sort_array(vec![]) == vec![]);
        assert!(sort_array(vec![5]) == vec![5]);
        assert!(sort_array(vec![2, 4, 3, 0, 1, 5]) == vec![0, 1, 2, 3, 4, 5]);
        assert!(sort_array(vec![2, 4, 3, 0, 1, 5, 6]) == vec![6, 5, 4, 3, 2, 1, 0]);
        assert!(sort_array(vec![2, 1]) == vec![1, 2]);
        assert!(sort_array(vec![15, 42, 87, 32, 11, 0]) == vec![0, 11, 15, 32, 42, 87]);
        assert!(sort_array(vec![21, 14, 23, 11]) == vec![23, 21, 14, 11]);
    }

}
