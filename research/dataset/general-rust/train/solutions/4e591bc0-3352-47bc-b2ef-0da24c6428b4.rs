use std::collections::HashMap;

pub fn two_sum(nums: Vec<i32>, target: i32) -> Option<(usize, usize)> {
    let mut seen = HashMap::new();

    for (i, &num) in nums.iter().enumerate() {
        let complement = target - num;

        if let Some(&j) = seen.get(&complement) {
            return Some((j, i));
        }

        seen.insert(num, i);
    }

    None
}
