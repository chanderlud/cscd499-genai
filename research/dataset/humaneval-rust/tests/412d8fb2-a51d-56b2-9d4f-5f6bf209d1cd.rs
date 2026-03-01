
#[cfg(test)]
mod tests {
    use super::*;

#[test]
    fn test_sort_numbers() {
        assert!(sort_numbers("".to_string()) == "".to_string());
        assert!(sort_numbers("three".to_string()) == "three".to_string());
        assert!(sort_numbers("three five nine".to_string()) == "three five nine");
        assert!(
            sort_numbers("five zero four seven nine eight".to_string())
                == "zero four five seven eight nine".to_string()
        );
        assert!(
            sort_numbers("six five four three two one zero".to_string())
                == "zero one two three four five six".to_string()
        );
    }

}
