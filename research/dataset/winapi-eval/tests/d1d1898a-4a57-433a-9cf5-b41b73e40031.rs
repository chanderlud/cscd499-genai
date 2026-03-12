#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_veh_guarded_read_u32_happy_path() {
        let value = 0xDEADBEEF;
        let result = veh_guarded_read_u32(value).unwrap();
        assert_eq!(result, value);
    }

    #[test]
    fn test_veh_guarded_read_u32_zero() {
        let value = 0u32;
        let result = veh_guarded_read_u32(value).unwrap();
        assert_eq!(result, value);
    }

    #[test]
    fn test_veh_guarded_read_u32_max_u32() {
        let value = u32::MAX;
        let result = veh_guarded_read_u32(value).unwrap();
        assert_eq!(result, value);
    }

    #[test]
    fn test_veh_guarded_read_u32_cafebabe() {
        let value = 0xCAFEBABE;
        let result = veh_guarded_read_u32(value).unwrap();
        assert_eq!(result, value);
    }

    #[test]
    fn test_veh_guarded_read_u32_error_propagation() {
        let value = 0x12345678;
        let result = veh_guarded_read_u32(value);
        assert!(result.is_ok());
    }
}
