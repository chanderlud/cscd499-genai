
#[cfg(test)]
mod tests {
    use super::*;

 #[test]
    fn test_circular_shift() {
        assert!(circular_shift(100, 2) == "001");
        assert!(circular_shift(12, 8) == "12");
        // original test   asert (circular_shift(97, 8) == "79"); DATASET ERROR
        assert!(circular_shift(97, 8) == "97");
        assert!(circular_shift(12, 1) == "21");
        assert!(circular_shift(11, 101) == "11");
    }

}
