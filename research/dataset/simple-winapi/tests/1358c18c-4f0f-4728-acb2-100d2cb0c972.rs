// Auto-generated tests for: 1358c18c-4f0f-4728-acb2-100d2cb0c972.md
// Model: arcee-ai/trinity-large-preview:free
// Extraction: rust

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_single_thread_single_phase() {
        let result = barrier_phased_counter(1, 1).unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_single_thread_multiple_phases() {
        let result = barrier_phased_counter(1, 100).unwrap();
        assert_eq!(result, 100);
    }

    #[test]
    fn test_multiple_threads_single_phase() {
        let result = barrier_phased_counter(4, 1).unwrap();
        assert_eq!(result, 4);
    }

    #[test]
    fn test_multiple_threads_multiple_phases() {
        let result = barrier_phased_counter(4, 1000).unwrap();
        assert_eq!(result, 4000);
    }

    #[test]
    fn test_zero_threads() {
        let result = barrier_phased_counter(0, 10).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_zero_phases() {
        let result = barrier_phased_counter(5, 0).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_zero_threads_and_phases() {
        let result = barrier_phased_counter(0, 0).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_large_number_of_phases() {
        let result = barrier_phased_counter(10, 1_000_000).unwrap();
        assert_eq!(result, 10_000_000);
    }

    #[test]
    fn test_large_number_of_threads() {
        let result = barrier_phased_counter(100, 10).unwrap();
        assert_eq!(result, 1000);
    }

    #[test]
    fn test_large_number_of_threads_and_phases() {
        let result = barrier_phased_counter(50, 2000).unwrap();
        assert_eq!(result, 100_000);
    }
}
