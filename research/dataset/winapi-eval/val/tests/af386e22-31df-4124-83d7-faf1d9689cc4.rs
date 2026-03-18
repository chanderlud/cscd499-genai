#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fiber_round_robin_basic() -> Result<()> {
        let trace = fiber_round_robin(3, 2)?;
        assert_eq!(trace, vec![0, 1, 2, 0, 1, 2]);
        Ok(())
    }

    #[test]
    fn test_fiber_round_robin_single_fiber() -> Result<()> {
        let trace = fiber_round_robin(1, 5)?;
        assert_eq!(trace, vec![0, 0, 0, 0, 0]);
        Ok(())
    }

    #[test]
    fn test_fiber_round_robin_single_iteration() -> Result<()> {
        let trace = fiber_round_robin(4, 1)?;
        assert_eq!(trace, vec![0, 1, 2, 3]);
        Ok(())
    }

    #[test]
    fn test_fiber_round_robin_zero_fibers() -> Result<()> {
        let trace = fiber_round_robin(0, 10)?;
        assert_eq!(trace, Vec::<u32>::new());
        Ok(())
    }

    #[test]
    fn test_fiber_round_robin_zero_iterations() -> Result<()> {
        let trace = fiber_round_robin(5, 0)?;
        assert_eq!(trace, Vec::<u32>::new());
        Ok(())
    }

    #[test]
    fn test_fiber_round_robin_large() -> Result<()> {
        let n_fibers = 10;
        let iters = 3;
        let trace = fiber_round_robin(n_fibers, iters)?;
        let expected: Vec<u32> = (0..iters).flat_map(|_| 0..n_fibers).collect();
        assert_eq!(trace, expected);
        Ok(())
    }
}
