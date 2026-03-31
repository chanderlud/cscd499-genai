#[cfg(test)]
mod tests {
    use super::milliseconds_since_boot;
    use std::{thread::sleep, time::Duration};

    #[test]
    fn back_to_back_calls_are_non_decreasing() {
        let first = milliseconds_since_boot();
        let second = milliseconds_since_boot();
        assert!(second >= first);
    }

    #[test]
    fn value_increases_after_a_short_sleep() {
        let before = milliseconds_since_boot();
        sleep(Duration::from_millis(64));
        let after = milliseconds_since_boot();
        assert!(after > before);
    }
}
