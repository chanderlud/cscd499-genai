
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f() {
        assert!(f(5) == vec![1, 2, 6, 24, 15]);
        assert!(f(7) == vec![1, 2, 6, 24, 15, 720, 28]);
        assert!(f(1) == vec![1]);
        assert!(f(3) == vec![1, 2, 6]);
    }

}
