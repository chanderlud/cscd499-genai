#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_bounded_queue_stress_basic() {
        let n_items = 10_000;
        let capacity = 64;
        let producers = 4;
        let consumers = 4;
        let result = bounded_queue_stress(n_items, capacity, producers, consumers).unwrap();
        assert_eq!(result.len(), n_items as usize);
        let set: HashSet<_> = result.into_iter().collect();
        assert_eq!(set.len(), n_items as usize);
    }

    #[test]
    fn test_bounded_queue_stress_small() {
        let n_items = 100;
        let capacity = 10;
        let producers = 2;
        let consumers = 2;
        let result = bounded_queue_stress(n_items, capacity, producers, consumers).unwrap();
        assert_eq!(result.len(), n_items as usize);
        let set: HashSet<_> = result.into_iter().collect();
        assert_eq!(set.len(), n_items as usize);
    }

    #[test]
    fn test_bounded_queue_stress_single_producer_consumer() {
        let n_items = 1000;
        let capacity = 50;
        let producers = 1;
        let consumers = 1;
        let result = bounded_queue_stress(n_items, capacity, producers, consumers).unwrap();
        assert_eq!(result.len(), n_items as usize);
        let set: HashSet<_> = result.into_iter().collect();
        assert_eq!(set.len(), n_items as usize);
    }

    #[test]
    fn test_bounded_queue_stress_capacity_one() {
        let n_items = 500;
        let capacity = 1;
        let producers = 2;
        let consumers = 2;
        let result = bounded_queue_stress(n_items, capacity, producers, consumers).unwrap();
        assert_eq!(result.len(), n_items as usize);
        let set: HashSet<_> = result.into_iter().collect();
        assert_eq!(set.len(), n_items as usize);
    }

    #[test]
    fn test_bounded_queue_stress_zero_items() {
        let n_items = 0;
        let capacity = 10;
        let producers = 2;
        let consumers = 2;
        let result = bounded_queue_stress(n_items, capacity, producers, consumers).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_bounded_queue_stress_zero_capacity() {
        let n_items = 100;
        let capacity = 0;
        let producers = 2;
        let consumers = 2;
        let result = bounded_queue_stress(n_items, capacity, producers, consumers);
        assert!(result.is_err());
    }

    #[test]
    fn test_bounded_queue_stress_zero_producers() {
        let n_items = 100;
        let capacity = 10;
        let producers = 0;
        let consumers = 2;
        let result = bounded_queue_stress(n_items, capacity, producers, consumers);
        assert!(result.is_err());
    }

    #[test]
    fn test_bounded_queue_stress_zero_consumers() {
        let n_items = 100;
        let capacity = 10;
        let producers = 2;
        let consumers = 0;
        let result = bounded_queue_stress(n_items, capacity, producers, consumers);
        assert!(result.is_err());
    }
}
