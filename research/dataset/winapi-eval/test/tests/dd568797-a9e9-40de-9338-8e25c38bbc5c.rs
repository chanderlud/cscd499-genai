#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_reset_event_starts_nonsignaled_then_toggles() {
        let report = probe_manual_reset_event(false).unwrap();

        assert_eq!(report.initial_sample_signaled, false);
        assert_eq!(report.after_set_sample_signaled, true);
        assert_eq!(report.after_reset_sample_signaled, false);
    }

    #[test]
    fn manual_reset_event_can_start_signaled() {
        let report = probe_manual_reset_event(true).unwrap();

        assert_eq!(report.initial_sample_signaled, true);
        assert_eq!(report.after_set_sample_signaled, true);
        assert_eq!(report.after_reset_sample_signaled, false);
    }

    #[test]
    fn repeated_calls_are_independent() {
        let first = probe_manual_reset_event(false).unwrap();
        let second = probe_manual_reset_event(false).unwrap();

        assert_eq!(first, second);
    }
}
