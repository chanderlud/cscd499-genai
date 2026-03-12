#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_com_init_mta_success() {
        let _com = com_init(
            windows::Win32::System::Com::COINIT_MULTITHREADED
                .0
                .try_into()
                .unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn test_com_init_sta_success() {
        let _com = com_init(
            windows::Win32::System::Com::COINIT_APARTMENTTHREADED
                .0
                .try_into()
                .unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn test_com_init_invalid_flag_returns_error() {
        let result = com_init(0x12345678);
        assert!(result.is_err());
    }

    #[test]
    fn test_com_init_already_initialized_returns_error() {
        let _com1 = com_init(
            windows::Win32::System::Com::COINIT_MULTITHREADED
                .0
                .try_into()
                .unwrap(),
        )
        .unwrap();
        let result = com_init(
            windows::Win32::System::Com::COINIT_MULTITHREADED
                .0
                .try_into()
                .unwrap(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_com_guard_drop_uninitializes_com() {
        {
            let _com = com_init(
                windows::Win32::System::Com::COINIT_MULTITHREADED
                    .0
                    .try_into()
                    .unwrap(),
            )
            .unwrap();
        }
        let result = com_init(
            windows::Win32::System::Com::COINIT_MULTITHREADED
                .0
                .try_into()
                .unwrap(),
        );
        assert!(result.is_ok());
    }
}
