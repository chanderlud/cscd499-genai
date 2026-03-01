
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digits() {
        assert_eq!(digits(5), 5);
        assert_eq!(digits(54), 5);
        assert_eq!(digits(120), 1);
        assert_eq!(digits(5014), 5);
        assert_eq!(digits(98765), 315);
        assert_eq!(digits(5576543), 2625);
        assert_eq!(digits(2468), 0);
    }

}
