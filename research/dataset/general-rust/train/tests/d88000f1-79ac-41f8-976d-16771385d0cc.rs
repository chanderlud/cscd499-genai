#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_rows_and_ignores_empty_lines() {
        let rows = ["north-lamp|3|-4", "   ", "east|2|5"];
        let out = parse_beacon_log(&rows).unwrap();

        assert_eq!(
            out,
            vec![
                Beacon {
                    name: "north-lamp",
                    count: 3,
                    delta: -4,
                    power: 12
                },
                Beacon {
                    name: "east",
                    count: 2,
                    delta: 5,
                    power: 10
                }
            ]
        );
    }

    #[test]
    fn rejects_malformed_rows() {
        let rows = ["north|7"];
        assert_eq!(parse_beacon_log(&rows), Err(LogError::Malformed("north|7")));
    }

    #[test]
    fn rejects_invalid_names() {
        let rows = ["North|3|1"];
        assert_eq!(parse_beacon_log(&rows), Err(LogError::InvalidName("North")));
    }

    #[test]
    fn rejects_duplicate_names() {
        let rows = ["east|1|2", "east|9|-1"];
        assert_eq!(
            parse_beacon_log(&rows),
            Err(LogError::DuplicateName("east"))
        );
    }

    #[test]
    fn rejects_overflow() {
        let rows = ["huge|4294967295|-2"];
        assert_eq!(
            parse_beacon_log(&rows),
            Err(LogError::Overflow("huge|4294967295|-2"))
        );
    }

    #[test]
    fn handles_i32_min_delta() {
        let rows = ["deep|1|-2147483648"];
        let out = parse_beacon_log(&rows).unwrap();
        assert_eq!(out[0].power, 2_147_483_648);
    }
}