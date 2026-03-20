#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngExt;

    fn random_alpha_string(len: usize) -> String {
        let mut rng = rand::rng();
        let mut out = String::with_capacity(len);

        for _ in 0..len {
            let upper = rng.random_bool(0.5);
            let c = if upper {
                rng.random_range(b'A'..=b'Z') as char
            } else {
                rng.random_range(b'a'..=b'z') as char
            };
            out.push(c);
        }
        out
    }

    #[test]
    fn test_decode_empty_string() {
        let s = "";
        let enc = encode_shift(s);
        let dec = decode_shift(&enc);
        assert_eq!(dec, s);
    }

    #[test]
    fn test_decode_single_char_all_letters() {
        // Lowercase
        for b in b'a'..=b'z' {
            let s = (b as char).to_string();
            let enc = encode_shift(&s);
            let dec = decode_shift(&enc);
            assert_eq!(dec, s, "failed on lowercase char {}", s);
        }

        // Uppercase
        for b in b'A'..=b'Z' {
            let s = (b as char).to_string();
            let enc = encode_shift(&s);
            let dec = decode_shift(&enc);
            assert_eq!(dec, s, "failed on uppercase char {}", s);
        }
    }

    #[test]
    fn test_decode_known_strings_roundtrip() {
        // Deterministic cases, including wrap-around candidates and mixed case.
        let cases = [
            "abc",
            "xyz",
            "ABC",
            "XYZ",
            "aAzZ",
            "zZaA",
            "HelloWorld",
            "RustLANG",
            "aaaaaaaaaa",
            "ZZZZzzzz",
        ];

        for s in cases {
            let enc = encode_shift(s);
            let dec = decode_shift(&enc);
            assert_eq!(dec, s, "roundtrip failed for input {:?}", s);
        }
    }

    #[test]
    fn test_decode_encode_random_alpha_only() {
        let mut rng = rand::rng();

        for _ in 0..500 {
            let len = rng.random_range(0..=64);
            let s = random_alpha_string(len);

            let encoded = encode_shift(&s);
            let decoded = decode_shift(&encoded);

            assert_eq!(decoded, s);
            // Extra sanity: decoding shouldn't change length for ASCII alpha inputs.
            assert_eq!(decoded.len(), s.len());
        }
    }

    #[test]
    fn test_roundtrip_long_input() {
        // Long-ish input to catch any weird indexing / overflow / performance bugs.
        let s = "aBcDeFgHiJkLmNoPqRsTuVwXyZ".repeat(1000);
        let enc = encode_shift(&s);
        let dec = decode_shift(&enc);
        assert_eq!(dec, s);
    }
}