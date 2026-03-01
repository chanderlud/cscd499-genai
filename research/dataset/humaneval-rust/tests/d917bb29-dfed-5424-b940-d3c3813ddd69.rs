
#[cfg(test)]
mod tests {
    use super::*;

#[test]
    fn test_rescale_to_unit() {
        assert!(rescale_to_unit(vec![2.0, 49.9]) == [0.0, 1.0]);
        assert!(rescale_to_unit(vec![100.0, 49.9]) == [1.0, 0.0]);
        assert!(rescale_to_unit(vec![1.0, 2.0, 3.0, 4.0, 5.0]) == [0.0, 0.25, 0.5, 0.75, 1.0]);
        assert!(rescale_to_unit(vec![2.0, 1.0, 5.0, 3.0, 4.0]) == [0.25, 0.0, 1.0, 0.5, 0.75]);
        assert!(rescale_to_unit(vec![12.0, 11.0, 15.0, 13.0, 14.0]) == [0.25, 0.0, 1.0, 0.5, 0.75]);
    }

}
