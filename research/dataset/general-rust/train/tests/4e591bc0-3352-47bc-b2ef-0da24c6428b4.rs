#[cfg(test)]
mod tests {
    use super::two_sum;

    fn assert_valid(nums: Vec<i32>, target: i32, ans: Option<(usize, usize)>) {
        match ans {
            None => {
                // Verify no solution exists (O(n^2) check is fine in tests)
                for i in 0..nums.len() {
                    for j in (i + 1)..nums.len() {
                        assert_ne!(
                            nums[i] + nums[j],
                            target,
                            "Expected None, but found a valid pair ({i},{j})"
                        );
                    }
                }
            }
            Some((i, j)) => {
                assert!(i < nums.len(), "Index i out of bounds: {i}");
                assert!(j < nums.len(), "Index j out of bounds: {j}");
                assert_ne!(i, j, "Indices must be distinct");
                assert_eq!(
                    nums[i] + nums[j],
                    target,
                    "Returned indices do not sum to target"
                );
            }
        }
    }

    #[test]
    fn basic_example() {
        let nums = vec![2, 7, 11, 15];
        let target = 9;
        let ans = two_sum(nums.clone(), target);
        assert_valid(nums, target, ans);
    }

    #[test]
    fn duplicate_values() {
        let nums = vec![3, 3];
        let target = 6;
        let ans = two_sum(nums.clone(), target);
        assert_valid(nums, target, ans);
    }

    #[test]
    fn negative_numbers() {
        let nums = vec![-1, -2, -3, -4, -5];
        let target = -8; // -3 + -5
        let ans = two_sum(nums.clone(), target);
        assert_valid(nums, target, ans);
    }

    #[test]
    fn mixed_numbers() {
        let nums = vec![3, 2, 4];
        let target = 6;
        let ans = two_sum(nums.clone(), target);
        assert_valid(nums, target, ans);
    }

    #[test]
    fn no_solution() {
        let nums = vec![1, 2, 3];
        let target = 7;
        let ans = two_sum(nums.clone(), target);
        assert_valid(nums, target, ans);
        assert!(ans.is_none(), "Expected None for no-solution case");
    }

    #[test]
    fn larger_input_sanity() {
        // Construct a large vector where we know exactly one solution exists.
        // nums = [0, 1, 2, ..., 9999], target = 19997 -> (9998, 9999)
        let n = 10_000;
        let nums: Vec<i32> = (0..n).collect();
        let target = (n - 2) + (n - 1); // 9998 + 9999 = 19997
        let ans = two_sum(nums.clone(), target);
        assert_valid(nums, target, ans);
    }

    #[test]
    fn solution_uses_distinct_indices_with_same_value() {
        // Two identical values in different positions should be allowed.
        let nums = vec![1, 5, 1];
        let target = 2;
        let ans = two_sum(nums.clone(), target);
        assert_valid(nums, target, ans);
    }
}