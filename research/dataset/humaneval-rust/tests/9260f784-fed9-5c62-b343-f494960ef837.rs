
#[cfg(test)]
mod tests {
    use super::*;

#[test]
    fn test_how_many_times() {
        assert!(how_many_times("".to_string(), "x".to_string()) == 0);
        assert!(how_many_times("xyxyxyx".to_string(), "x".to_string()) == 4);
        assert!(how_many_times("cacacacac".to_string(), "cac".to_string()) == 4);
        assert!(how_many_times("john doe".to_string(), "john".to_string()) == 1);
    }


}
