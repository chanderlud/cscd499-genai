#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wakes_every_worker_and_stays_signaled() {
        let report = run_stage_red_light_rehearsal(4).unwrap();

        assert!(report.initial_probe_timed_out);
        assert_eq!(report.awakened_workers, 4);
        assert!(report.post_signal_probe_passed);
    }

    #[test]
    fn handles_zero_workers_deterministically() {
        let report = run_stage_red_light_rehearsal(0).unwrap();

        assert!(report.initial_probe_timed_out);
        assert_eq!(report.awakened_workers, 0);
        assert!(report.post_signal_probe_passed);
    }

    #[test]
    fn single_worker_case_is_also_valid() {
        let report = run_stage_red_light_rehearsal(1).unwrap();

        assert!(report.initial_probe_timed_out);
        assert_eq!(report.awakened_workers, 1);
        assert!(report.post_signal_probe_passed);
    }
}