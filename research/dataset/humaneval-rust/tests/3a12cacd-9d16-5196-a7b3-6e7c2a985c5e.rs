
#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_anti_shuffle() {
        assert!(anti_shuffle("Hi") == "Hi".to_string());
        assert!(anti_shuffle("hello") == "ehllo".to_string());
        assert!(anti_shuffle("number") == "bemnru".to_string());
        assert!(anti_shuffle("abcd") == "abcd".to_string());
        assert!(anti_shuffle("Hello World!!!") == "Hello !!!Wdlor".to_string());
        assert!(anti_shuffle("") == "".to_string());
        assert!(
            anti_shuffle("Hi. My name is Mister Robot. How are you?")
                == ".Hi My aemn is Meirst .Rboot How aer ?ouy".to_string()
        );
    }

}
