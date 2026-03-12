// Auto-generated tests for: d18aa844-1596-4d72-be83-a0de01a5c58f.md
// Model: arcee-ai/trinity-large-preview:free
// Extraction: rust

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_fls_destructor_count_basic() -> io::Result<()> {
        let count = fls_destructor_count(8)?;
        assert_eq!(count, 8);
        Ok(())
    }

    #[test]
    fn test_fls_destructor_count_zero_threads() -> io::Result<()> {
        let count = fls_destructor_count(0)?;
        assert_eq!(count, 0);
        Ok(())
    }

    #[test]
    fn test_fls_destructor_count_one_thread() -> io::Result<()> {
        let count = fls_destructor_count(1)?;
        assert_eq!(count, 1);
        Ok(())
    }

    #[test]
    fn test_fls_destructor_count_large_number() -> io::Result<()> {
        let count = fls_destructor_count(1000)?;
        assert_eq!(count, 1000);
        Ok(())
    }

    #[test]
    fn test_fls_destructor_count_max_usize() -> io::Result<()> {
        let count = fls_destructor_count(usize::MAX)?;
        assert_eq!(count, usize::MAX as i32);
        Ok(())
    }
}
