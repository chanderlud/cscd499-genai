#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tp_timer_signal_fires_before_timeout() {
        let fired = tp_timer_signal(50, 200).expect("tp_timer_signal failed");
        assert!(fired, "Timer should fire before timeout");
    }

    #[test]
    fn test_tp_timer_signal_does_not_fire_within_timeout() {
        let fired = tp_timer_signal(200, 50).expect("tp_timer_signal failed");
        assert!(!fired, "Timer should not fire within timeout");
    }

    #[test]
    fn test_tp_timer_signal_zero_due_time_triggers_immediately() {
        let fired = tp_timer_signal(0, 100).expect("tp_timer_signal failed");
        assert!(fired, "Timer with zero due time should fire immediately");
    }

    #[test]
    fn test_tp_timer_signal_far_future_due_time_times_out() {
        let fired = tp_timer_signal(u32::MAX, 1000).expect("tp_timer_signal failed");
        assert!(
            !fired,
            "A timer due in ~49.7 days should not fire within 1 second"
        );
    }

    #[test]
    fn test_tp_timer_signal_timeout_zero_returns_false() {
        let fired = tp_timer_signal(50, 0).expect("tp_timer_signal failed");
        assert!(!fired, "Timer with zero timeout should not fire");
    }

    #[test]
    fn test_tp_timer_signal_due_time_well_below_timeout() {
        let fired = tp_timer_signal(100, 300).expect("tp_timer_signal failed");
        assert!(fired, "Timer with comfortable slack should fire");
    }

    #[test]
    fn test_tp_timer_signal_due_time_well_above_timeout() {
        let fired = tp_timer_signal(300, 100).expect("tp_timer_signal failed");
        assert!(!fired, "Timer with comfortable slack should not fire");
    }

    #[test]
    fn test_tp_timer_signal_short_delay_fires() {
        let fired = tp_timer_signal(10, 100).expect("tp_timer_signal failed");
        assert!(fired, "Timer with short delay should fire");
    }

    #[test]
    fn test_tp_timer_signal_very_short_delay_fires() {
        let fired = tp_timer_signal(1, 50).expect("tp_timer_signal failed");
        assert!(fired, "Timer with very short delay should fire");
    }
}