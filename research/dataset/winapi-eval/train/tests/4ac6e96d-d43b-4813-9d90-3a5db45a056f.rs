#[cfg(test)]
#[cfg(windows)]
mod tests {
    use super::*;

    fn run_on_fresh_thread<F>(f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        std::thread::spawn(f).join().unwrap();
    }

    #[test]
    fn test_com_init_mta_success() {
        run_on_fresh_thread(|| {
            let _com = com_init(
                windows::Win32::System::Com::COINIT_MULTITHREADED
                    .0
                    .try_into()
                    .unwrap(),
            )
                .unwrap();
        });
    }

    #[test]
    fn test_com_init_sta_success() {
        run_on_fresh_thread(|| {
            let _com = com_init(
                windows::Win32::System::Com::COINIT_APARTMENTTHREADED
                    .0
                    .try_into()
                    .unwrap(),
            )
                .unwrap();
        });
    }

    #[test]
    fn test_com_init_invalid_flag_returns_error() {
        run_on_fresh_thread(|| {
            // Uses a bit outside the documented COINIT flag set.
            let result = com_init(0x10);
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_com_init_same_mode_twice_succeeds() {
        run_on_fresh_thread(|| {
            let _com1 = com_init(
                windows::Win32::System::Com::COINIT_MULTITHREADED
                    .0
                    .try_into()
                    .unwrap(),
            )
                .unwrap();

            let _com2 = com_init(
                windows::Win32::System::Com::COINIT_MULTITHREADED
                    .0
                    .try_into()
                    .unwrap(),
            )
                .unwrap();
        });
    }

    #[test]
    fn test_com_init_changed_mode_returns_error() {
        run_on_fresh_thread(|| {
            let _com = com_init(
                windows::Win32::System::Com::COINIT_MULTITHREADED
                    .0
                    .try_into()
                    .unwrap(),
            )
                .unwrap();

            let result = com_init(
                windows::Win32::System::Com::COINIT_APARTMENTTHREADED
                    .0
                    .try_into()
                    .unwrap(),
            );

            assert!(result.is_err());
        });
    }

    #[test]
    fn test_com_guard_drop_uninitializes_com() {
        run_on_fresh_thread(|| {
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
        });
    }
}
