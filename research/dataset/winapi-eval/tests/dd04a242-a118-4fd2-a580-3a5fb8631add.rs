#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_iocp_post_and_drain_basic() -> Result<()> {
        let keys = [1, 2, 3];
        let seen = iocp_post_and_drain(&keys, 1000)?;
        assert_eq!(seen.len(), keys.len());
        let seen_set: HashSet<_> = seen.into_iter().collect();
        let expected_set: HashSet<_> = keys.iter().cloned().collect();
        assert_eq!(seen_set, expected_set);
        Ok(())
    }

    #[test]
    fn test_iocp_post_and_drain_empty() -> Result<()> {
        let seen = iocp_post_and_drain(&[], 1000)?;
        assert!(seen.is_empty());
        Ok(())
    }

    #[test]
    fn test_iocp_post_and_drain_single() -> Result<()> {
        let seen = iocp_post_and_drain(&[42], 1000)?;
        assert_eq!(seen, [42]);
        Ok(())
    }

    #[test]
    fn test_iocp_post_and_drain_timeout() -> Result<()> {
        let keys = [1, 2];
        let seen = iocp_post_and_drain(&keys, 0)?;
        assert_eq!(seen.len(), 0);
        Ok(())
    }

    #[test]
    fn test_iocp_post_and_drain_large() -> Result<()> {
        let keys: Vec<_> = (0..100).collect();
        let seen = iocp_post_and_drain(&keys, 1000)?;
        assert_eq!(seen.len(), keys.len());
        let seen_set: HashSet<_> = seen.into_iter().collect();
        let expected_set: HashSet<_> = keys.into_iter().collect();
        assert_eq!(seen_set, expected_set);
        Ok(())
    }

    #[test]
    fn test_iocp_post_and_drain_duplicate_keys() -> Result<()> {
        let keys = [5, 5, 5];
        let seen = iocp_post_and_drain(&keys, 1000)?;
        assert_eq!(seen.len(), keys.len());
        assert!(seen.iter().all(|&k| k == 5));
        Ok(())
    }
}
