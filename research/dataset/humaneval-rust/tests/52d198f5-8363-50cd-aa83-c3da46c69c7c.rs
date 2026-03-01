
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_will_it_fly() {
        assert!(will_it_fly(vec![3, 2, 3], 9) == true);
        assert!(will_it_fly(vec![1, 2], 5) == false);
        assert!(will_it_fly(vec![3], 5) == true);
        assert!(will_it_fly(vec![3, 2, 3], 1) == false);
        assert!(will_it_fly(vec![1, 2, 3], 6) == false);
        assert!(will_it_fly(vec![5], 5) == true);
    }

}
