
#[cfg(test)]
mod tests {
    use super::*;

#[test]
    fn test_poly() {
        let mut rng = rand::thread_rng();
        let mut solution: f64;
        let mut ncoeff: i32;
        for _ in 0..100 {
            ncoeff = 2 * (1 + rng.gen_range(0, 4));
            let mut coeffs = vec![];
            for _ in 0..ncoeff {
                let coeff = -10 + rng.gen_range(0, 21);
                if coeff == 0 {
                    coeffs.push(1.0);
                } else {
                    coeffs.push(coeff as f64);
                }
            }
            solution = find_zero(&coeffs);
            assert!(poly(&coeffs, solution).abs() < 1e-3);
        }
    }

}
