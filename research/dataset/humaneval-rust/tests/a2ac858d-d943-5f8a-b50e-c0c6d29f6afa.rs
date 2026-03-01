
#[cfg(test)]
mod tests {
    use super::*;

#[test]
    fn test_flip_case() {
        assert!(flip_case("".to_string()) == "".to_string());
        assert!(flip_case("Hello!".to_string()) == "hELLO!".to_string());
        assert!(
            flip_case("These violent delights have violent ends".to_string())
                == "tHESE VIOLENT DELIGHTS HAVE VIOLENT ENDS".to_string()
        );
    }

}
