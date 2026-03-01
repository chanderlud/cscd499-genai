
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digitSum() {
        assert!(digitSum("") == 0);
        assert!(digitSum("abAB") == 131);
        assert!(digitSum("abcCd") == 67);
        assert!(digitSum("helloE") == 69);
        assert!(digitSum("woArBld") == 131);
        assert!(digitSum("aAaaaXa") == 153);
        assert!(digitSum(" How are yOu?") == 151);
        assert!(digitSum("You arE Very Smart") == 327);
    }


}
