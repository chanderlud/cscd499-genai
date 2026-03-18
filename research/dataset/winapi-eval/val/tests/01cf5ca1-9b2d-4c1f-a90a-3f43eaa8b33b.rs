#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn unique_name(base: &str) -> String {
        format!(
            "Local\\{}_{}_{}",
            base,
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        )
    }

    #[test]
    fn test_named_mapping_write_read_basic() {
        let name = unique_name("basic_test");
        let size = 4096;
        let offset = 10;
        let data = b"hello";

        let result = named_mapping_write_read(&name, size, offset, data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_named_mapping_write_read_empty_data() {
        let name = unique_name("empty_data_test");
        let size = 4096;
        let offset = 0;
        let data: &[u8] = b"";

        let result = named_mapping_write_read(&name, size, offset, data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_named_mapping_write_read_boundary_offset() {
        let name = unique_name("boundary_offset_test");
        let size = 4096;
        let offset = size - 1;
        let data = b"x";

        let result = named_mapping_write_read(&name, size, offset, data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_named_mapping_write_read_offset_out_of_bounds() {
        let name = unique_name("oob_offset_test");
        let size = 4096;
        let offset = size + 1;
        let data = b"too_far";

        let result = named_mapping_write_read(&name, size, offset, data);
        assert!(result.is_err());
    }

    #[test]
    fn test_named_mapping_write_read_data_extends_past_end() {
        let name = unique_name("overflow_write_test");
        let size = 4096;
        let offset = size - 2;
        let data = b"abc";

        let result = named_mapping_write_read(&name, size, offset, data);
        assert!(result.is_err());
    }

    #[test]
    fn test_named_mapping_write_read_zero_size() {
        let name = unique_name("zero_size_test");
        let size = 0;
        let offset = 0;
        let data = b"invalid";

        let result = named_mapping_write_read(&name, size, offset, data);
        assert!(result.is_err());
    }

    #[test]
    fn test_named_mapping_write_read_fresh_mapping() {
        let name = unique_name("fresh_mapping_test");
        let size = 4096;
        let offset = 10;
        let data = b"should_be_created";

        let result = named_mapping_write_read(&name, size, offset, data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_named_mapping_write_read_long_name() {
        let name = unique_name(
            "this_is_a_very_long_mapping_name_that_exceeds_normal_length_but_should_still_work",
        );
        let size = 4096;
        let offset = 100;
        let data = b"long_name_test";

        let result = named_mapping_write_read(&name, size, offset, data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_named_mapping_write_read_multiple_bytes() {
        let name = unique_name("multiple_bytes_test");
        let size = 4096;
        let offset = 100;
        let data = b"hello world this is a test";

        let result = named_mapping_write_read(&name, size, offset, data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_named_mapping_write_read_zero_offset() {
        let name = unique_name("zero_offset_test");
        let size = 4096;
        let offset = 0;
        let data = b"start_at_zero";

        let result = named_mapping_write_read(&name, size, offset, data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_named_mapping_write_read_large_data() {
        let name = unique_name("large_data_test");
        let size = 4096;
        let offset = 0;
        let data = vec![b'a'; 4096];

        let result = named_mapping_write_read(&name, size, offset, &data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), data);
    }
}
