#[cfg(all(test, windows))]
mod tests {
    use super::align_mapping_offset;

    #[test]
    fn test_align_mapping_offset_zero_returns_correct_alignment() {
        let (aligned, delta, granularity) = align_mapping_offset(0).unwrap();
        assert_eq!(aligned, 0);
        assert_eq!(delta, 0);
        assert!(granularity > 0);
    }

    #[test]
    fn test_align_mapping_offset_one_aligns_to_granularity_boundary() {
        let (aligned, delta, granularity) = align_mapping_offset(1).unwrap();
        assert_eq!(aligned + delta, 1);
        assert_eq!(aligned % (granularity as u64), 0);
        assert!(delta < granularity as u64);
    }

    #[test]
    fn test_align_mapping_offset_exact_granularity_multiple() {
        let (aligned, delta, granularity) = align_mapping_offset(4096).unwrap();
        assert_eq!(aligned, 4096);
        assert_eq!(delta, 0);
        assert_eq!(granularity, 4096);
    }

    #[test]
    fn test_align_mapping_offset_below_granularity_boundary() {
        let (aligned, delta, granularity) = align_mapping_offset(4095).unwrap();
        assert_eq!(aligned, 0);
        assert_eq!(delta, 4095);
        assert_eq!(granularity, 4096);
    }

    #[test]
    fn test_align_mapping_offset_large_offset() {
        let (aligned, delta, granularity) = align_mapping_offset(123_456_789).unwrap();
        assert_eq!(aligned + delta, 123_456_789);
        assert_eq!(aligned % (granularity as u64), 0);
        assert!(delta < granularity as u64);
    }

    #[test]
    fn test_align_mapping_offset_maximum_u64_offset() {
        let (aligned, delta, granularity) = align_mapping_offset(u64::MAX).unwrap();
        assert_eq!(aligned + delta, u64::MAX);
        assert_eq!(aligned % (granularity as u64), 0);
        assert!(delta < granularity as u64);
    }

    #[test]
    fn test_align_mapping_offset_negative_not_applicable() {
        // u64 is unsigned, so negative values are not applicable
        // This test serves as documentation
    }
}
