#[cfg(test)]
mod tests {
    use super::current_system_time_100ns;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn returns_a_nonzero_timestamp() {
        let value = current_system_time_100ns();
        assert!(value > 0);
    }

    #[test]
    fn consecutive_calls_are_non_decreasing() {
        let first = current_system_time_100ns();
        let second = current_system_time_100ns();
        assert!(second >= first);
    }

    #[test]
    fn timestamp_advances_after_sleep() {
        let first = current_system_time_100ns();
        thread::sleep(Duration::from_millis(5));
        let second = current_system_time_100ns();
        assert!(second > first);
    }
}
