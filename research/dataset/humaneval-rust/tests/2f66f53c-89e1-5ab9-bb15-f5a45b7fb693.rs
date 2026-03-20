
#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngExt;

    #[test]
    fn test_add() {
        assert!(add(0, 1) == 1);
        assert!(add(1, 0) == 1);
        assert!(add(2, 3) == 5);
        assert!(add(5, 7) == 12);
        assert!(add(7, 5) == 12);
        for _ in 0..100 {
            let mut rng = rand::rng();
            let mut x: i32 = rng.random();
            x = x % 1000;
            let mut y: i32 = rng.random();
            y = y % 1000;

            assert!(add(x, y) == x + y);
        }
    }

}
