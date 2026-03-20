use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rune<'a> {
    pub glyph: &'a str,
    pub cost: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellWindow<'a> {
    pub total_cost: u32,
    pub runes: &'a [Rune<'a>],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpellError {
    ZeroBudget,
    EmptyGlyph { index: usize },
}

pub fn best_spell_window<'a>(
    runes: &'a [Rune<'a>],
    budget: u32,
) -> Result<SpellWindow<'a>, SpellError> {
    if budget == 0 {
        return Err(SpellError::ZeroBudget);
    }

    if let Some((index, _)) = runes
        .iter()
        .enumerate()
        .find(|(_, rune)| rune.glyph.is_empty())
    {
        return Err(SpellError::EmptyGlyph { index });
    }

    let mut counts: HashMap<&'a str, usize> = HashMap::new();
    let mut left = 0usize;
    let mut total_cost = 0u64;

    let mut best_start = 0usize;
    let mut best_len = 0usize;
    let mut best_cost = 0u32;

    for right in 0..runes.len() {
        let rune = &runes[right];
        total_cost += rune.cost as u64;
        *counts.entry(rune.glyph).or_insert(0) += 1;

        while total_cost > budget as u64 || counts.get(rune.glyph).copied().unwrap_or(0) > 1 {
            let left_rune = &runes[left];
            total_cost -= left_rune.cost as u64;

            if let Some(count) = counts.get_mut(left_rune.glyph) {
                *count -= 1;
                if *count == 0 {
                    counts.remove(left_rune.glyph);
                }
            }

            left += 1;
        }

        if left <= right {
            let current_len = right - left + 1;
            let current_cost = total_cost as u32;

            let should_replace = current_len > best_len
                || (current_len == best_len && (best_len == 0 || current_cost < best_cost));

            if should_replace {
                best_start = left;
                best_len = current_len;
                best_cost = current_cost;
            }
        }
    }

    Ok(SpellWindow {
        total_cost: best_cost,
        runes: &runes[best_start..best_start + best_len],
    })
}
