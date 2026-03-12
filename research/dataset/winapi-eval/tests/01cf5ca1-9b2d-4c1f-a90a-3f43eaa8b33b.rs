#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_named_mapping_write_read_basic() {
        let name = "Local\\basic_test";
        let size = 4096;
        let offset = 10;
        let data = b"hello";

        let result = named_mapping_write_read(name, size, offset, data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_named_mapping_write_read_empty_data() {
        let name = "Local\\empty_data_test";
        let size = 4096;
        let offset = 0;
        let data: &[u8] = b"";

        let result = named_mapping_write_read(name, size, offset, data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_named_mapping_write_read_boundary_offset() {
        let name = "Local\\boundary_offset_test";
        let size = 4096;
        let offset = size - 1;
        let data = b"x";

        let result = named_mapping_write_read(name, size, offset, data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_named_mapping_write_read_offset_out_of_bounds() {
        let name = "Local\\oob_offset_test";
        let size = 4096;
        let offset = size + 1;
        let data = b"too_far";

        let result = named_mapping_write_read(name, size, offset, data);
        assert!(result.is_err());
    }

    #[test]
    fn test_named_mapping_write_read_zero_size() {
        let name = "Local\\zero_size_test";
        let size = 0;
        let offset = 0;
        let data = b"invalid";

        let result = named_mapping_write_read(name, size, offset, data);
        assert!(result.is_err());
    }

    #[test]
    fn test_named_mapping_write_read_nonexistent_mapping() {
        let name = "Local\\nonexistent_map";
        let size = 4096;
        let offset = 10;
        let data = b"should_not_exist";

        let result = named_mapping_write_read(name, size, offset, data);
        assert!(result.is_err());
    }

    #[test]
    fn test_named_mapping_write_read_long_name() {
        let name = "Local\\this_is_a_very_long_mapping_name_that_exceeds_normal_length_but_should_still_work";
        let size = 4096;
        let offset = 100;
        let data = b"long_name_test";

        let result = named_mapping_write_read(name, size, offset, data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_named_mapping_write_read_multiple_bytes() {
        let name = "Local\\multiple_bytes_test";
        let size = 4096;
        let offset = 100;
        let data = b"hello world this is a test";

        let result = named_mapping_write_read(name, size, offset, data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_named_mapping_write_read_zero_offset() {
        let name = "Local\\zero_offset_test";
        let size = 4096;
        let offset = 0;
        let data = b"start_at_zero";

        let result = named_mapping_write_read(name, size, offset, data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_named_mapping_write_read_large_data() {
        let name = "Local\\large_data_test";
        let size = 4096;
        let offset = 0;
        let data = b"a".repeat(4096);

        let result = named_mapping_write_read(name, size, offset, &data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_named_mapping_write_read_partial_write() {
        let name = "Local\\partial_write_test";
        let size = 4096;
        let offset = 100;
        let data = b"partial";

        let result = named_mapping_write_read(name, size, offset, data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), data);
    }
}
