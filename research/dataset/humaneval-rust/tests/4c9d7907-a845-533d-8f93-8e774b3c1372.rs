
#[cfg(test)]
mod tests {
    use super::*;

 #[test]
    fn test_car_race_collision() {
        assert!(car_race_collision(2) == 4);
        assert!(car_race_collision(3) == 9);
        assert!(car_race_collision(4) == 16);
        assert!(car_race_collision(8) == 64);
        assert!(car_race_collision(10) == 100);
    }

}
