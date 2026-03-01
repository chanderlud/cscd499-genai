
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_spaces() {
        assert_eq!(fix_spaces("Example"), "Example");
        assert_eq!(fix_spaces("Mudasir Hanif "), "Mudasir_Hanif_");
        assert_eq!(
            fix_spaces("Yellow Yellow  Dirty  Fellow"),
            "Yellow_Yellow__Dirty__Fellow"
        );
        assert_eq!(fix_spaces("Exa   mple"), "Exa-mple");
        assert_eq!(fix_spaces("   Exa 1 2 2 mple"), "-Exa_1_2_2_mple");
    }

}
