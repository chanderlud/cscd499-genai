#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tp_timer_signal_fires_before_timeout() {
        let fired = tp_timer_signal(50, 100).expect("tp_timer_signal failed");
        assert!(fired, "Timer should fire before timeout");
    }

    #[test]
    fn test_tp_timer_signal_does_not_fire_within_timeout() {
        let fired = tp_timer_signal(200, 100).expect("tp_timer_signal failed");
        assert!(!fired, "Timer should not fire within timeout");
    }

    #[test]
    fn test_tp_timer_signal_zero_due_time_triggers_immediately() {
        let fired = tp_timer_signal(0, 100).expect("tp_timer_signal failed");
        assert!(fired, "Timer with zero due time should fire immediately");
    }

    #[test]
    fn test_tp_timer_signal_max_due_time_within_timeout() {
        let fired = tp_timer_signal(u32::MAX, 1000).expect("tp_timer_signal failed");
        assert!(fired, "Timer with max due time should fire");
    }

    #[test]
    fn test_tp_timer_signal_timeout_zero_returns_false() {
        let fired = tp_timer_signal(50, 0).expect("tp_timer_signal failed");
        assert!(!fired, "Timer with zero timeout should not fire");
    }

    #[test]
    fn test_tp_timer_signal_due_time_equal_to_timeout() {
        let fired = tp_timer_signal(100, 100).expect("tp_timer_signal failed");
        assert!(fired, "Timer with due time equal to timeout should fire");
    }

    #[test]
    fn test_tp_timer_signal_due_time_just_below_timeout() {
        let fired = tp_timer_signal(99, 100).expect("tp_timer_signal failed");
        assert!(fired, "Timer with due time just below timeout should fire");
    }

    #[test]
    fn test_tp_timer_signal_due_time_just_above_timeout() {
        let fired = tp_timer_signal(101, 100).expect("tp_timer_signal failed");
        assert!(
            !fired,
            "Timer with due time just above timeout should not fire"
        );
    }

    #[test]
    fn test_tp_timer_signal_short_delay_fires() {
        let fired = tp_timer_signal(10, 50).expect("tp_timer_signal failed");
        assert!(fired, "Timer with short delay should fire");
    }

    #[test]
    fn test_tp_timer_signal_very_short_delay_fires() {
        let fired = tp_timer_signal(1, 10).expect("tp_timer_signal failed");
        assert!(fired, "Timer with very short delay should fire");
    }
}
