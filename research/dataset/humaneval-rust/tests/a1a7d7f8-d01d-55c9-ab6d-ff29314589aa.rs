
#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngExt;

    #[test]
    fn test_poly() {
        let mut rng = rand::rng();
        let mut solution: f64;
        let mut ncoeff: i32;
        for _ in 0..100 {
            ncoeff = 2 * (1 + rng.random_range(0..4));
            let mut coeffs = vec![];
            for _ in 0..ncoeff {
                let coeff = -10 + rng.random_range(0..21);
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

    fn find_zero(xs: &Vec<f64>) -> f64 {
        // If 0 is already a root, return it immediately.
        let f0 = poly(xs, 0.0);
        if f0.abs() < 1e-12 {
            return 0.0;
        }

        // Find a bracket [lo, hi] with opposite signs.
        let mut lo = -1.0f64;
        let mut hi = 1.0f64;
        let mut flo = poly(xs, lo);
        let mut fhi = poly(xs, hi);

        while flo.signum() == fhi.signum() {
            lo *= 2.0;
            hi *= 2.0;
            flo = poly(xs, lo);
            fhi = poly(xs, hi);
        }

        // Bisection: guaranteed to converge once signs differ.
        for _ in 0..200 {
            let mid = (lo + hi) / 2.0;
            let fmid = poly(xs, mid);

            if fmid.abs() < 1e-12 {
                return mid;
            }

            if flo.signum() == fmid.signum() {
                lo = mid;
                flo = fmid;
            } else {
                hi = mid;
            }
        }

        (lo + hi) / 2.0
    }
}
