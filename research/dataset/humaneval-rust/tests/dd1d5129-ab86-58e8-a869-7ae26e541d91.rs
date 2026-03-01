
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iscuber() {
        assert!(iscuber(1) == true);
        assert!(iscuber(2) == false);
        assert!(iscuber(-1) == true);
        assert!(iscuber(64) == true);
        assert!(iscuber(180) == false);
        assert!(iscuber(1000) == true);
        assert!(iscuber(0) == true);
        assert!(iscuber(1729) == false);
    }

}
