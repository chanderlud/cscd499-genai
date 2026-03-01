
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_even_odd() {
        assert!(add_even_odd(vec![4, 88]) == 88);
        assert!(add_even_odd(vec![4, 5, 6, 7, 2, 122]) == 122);
        assert!(add_even_odd(vec![4, 0, 6, 7]) == 0);
        assert!(add_even_odd(vec![4, 4, 6, 8]) == 12);
    }


}
