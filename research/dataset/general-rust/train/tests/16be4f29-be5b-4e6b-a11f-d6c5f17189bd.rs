#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_longest_valid_window() {
        let runes = [
            Rune {
                glyph: "moon",
                cost: 2,
            },
            Rune {
                glyph: "mist",
                cost: 1,
            },
            Rune {
                glyph: "ember",
                cost: 2,
            },
            Rune {
                glyph: "moon",
                cost: 1,
            },
        ];

        let result = best_spell_window(&runes, 5).unwrap();

        assert_eq!(result.total_cost, 4);
        assert_eq!(
            result.runes.iter().map(|r| r.glyph).collect::<Vec<_>>(),
            vec!["mist", "ember", "moon"]
        );
    }

    #[test]
    fn breaks_on_duplicate_glyph_inside_window() {
        let runes = [
            Rune {
                glyph: "sun",
                cost: 2,
            },
            Rune {
                glyph: "mist",
                cost: 3,
            },
            Rune {
                glyph: "sun",
                cost: 1,
            },
            Rune {
                glyph: "ember",
                cost: 2,
            },
        ];

        let result = best_spell_window(&runes, 5).unwrap();

        assert_eq!(result.total_cost, 3);
        assert_eq!(
            result.runes.iter().map(|r| r.glyph).collect::<Vec<_>>(),
            vec!["sun", "ember"]
        );
    }

    #[test]
    fn tie_breaks_by_lower_total_cost() {
        let runes = [
            Rune {
                glyph: "a",
                cost: 3,
            },
            Rune {
                glyph: "b",
                cost: 3,
            },
            Rune {
                glyph: "wall",
                cost: 10,
            },
            Rune {
                glyph: "c",
                cost: 1,
            },
            Rune {
                glyph: "d",
                cost: 1,
            },
        ];

        let result = best_spell_window(&runes, 6).unwrap();

        assert_eq!(result.total_cost, 2);
        assert_eq!(
            result.runes.iter().map(|r| r.glyph).collect::<Vec<_>>(),
            vec!["c", "d"]
        );
    }

    #[test]
    fn tie_breaks_by_earliest_when_length_and_cost_match() {
        let runes = [
            Rune {
                glyph: "oak",
                cost: 2,
            },
            Rune {
                glyph: "ash",
                cost: 2,
            },
            Rune {
                glyph: "wall",
                cost: 9,
            },
            Rune {
                glyph: "ivy",
                cost: 2,
            },
            Rune {
                glyph: "fern",
                cost: 2,
            },
        ];

        let result = best_spell_window(&runes, 4).unwrap();

        assert_eq!(result.total_cost, 4);
        assert_eq!(
            result.runes.iter().map(|r| r.glyph).collect::<Vec<_>>(),
            vec!["oak", "ash"]
        );
    }

    #[test]
    fn returns_empty_window_when_nothing_fits() {
        let runes = [
            Rune {
                glyph: "storm",
                cost: 10,
            },
            Rune {
                glyph: "frost",
                cost: 11,
            },
        ];

        let result = best_spell_window(&runes, 5).unwrap();

        assert_eq!(result.total_cost, 0);
        assert!(result.runes.is_empty());
    }

    #[test]
    fn errors_on_zero_budget() {
        let runes = [Rune {
            glyph: "sun",
            cost: 1,
        }];

        let err = best_spell_window(&runes, 0).unwrap_err();

        assert_eq!(err, SpellError::ZeroBudget);
    }

    #[test]
    fn errors_on_empty_glyph() {
        let runes = [
            Rune {
                glyph: "sun",
                cost: 1,
            },
            Rune { glyph: "", cost: 2 },
        ];

        let err = best_spell_window(&runes, 5).unwrap_err();

        assert_eq!(err, SpellError::EmptyGlyph { index: 1 });
    }

    #[test]
    fn preserves_borrowed_slice_identity() {
        let runes = [
            Rune {
                glyph: "a",
                cost: 1,
            },
            Rune {
                glyph: "b",
                cost: 1,
            },
            Rune {
                glyph: "a",
                cost: 1,
            },
        ];

        let result = best_spell_window(&runes, 2).unwrap();

        assert!(std::ptr::eq(result.runes.as_ptr(), runes.as_ptr()));
        assert_eq!(result.runes.len(), 2);
    }
}