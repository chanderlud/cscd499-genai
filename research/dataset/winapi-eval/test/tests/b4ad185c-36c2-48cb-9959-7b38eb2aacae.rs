#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_std_process_id() {
        assert_eq!(current_process_id(), std::process::id());
    }

    #[test]
    fn is_same_across_threads_in_one_process() {
        let pid_main = current_process_id();

        let pid_from_thread = std::thread::spawn(current_process_id)
            .join()
            .expect("thread should complete successfully");

        assert_eq!(pid_main, pid_from_thread);
    }

    #[test]
    fn process_id_is_nonzero() {
        assert_ne!(current_process_id(), 0);
    }
}
