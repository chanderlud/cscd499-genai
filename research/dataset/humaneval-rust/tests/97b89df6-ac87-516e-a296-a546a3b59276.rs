
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_matrix_triples() {
        assert_eq!(get_matrix_triples(5), 1);
        assert_eq!(get_matrix_triples(6), 4);
        assert_eq!(get_matrix_triples(10), 36);
        assert_eq!(get_matrix_triples(100), 53361);
    }

}
