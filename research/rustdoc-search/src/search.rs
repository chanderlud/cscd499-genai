use serde::{Deserialize, Serialize};

use crate::doc_parser::{DocItem, ItemKind, SearchIndex};

/// A single search result with relevance scoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The item's name.
    pub name: String,
    /// The kind of item.
    pub kind: ItemKind,
    /// Full module path.
    pub path: String,
    /// Item signature or declaration.
    pub signature: Option<String>,
    /// Documentation text.
    pub documentation: Option<String>,
    /// Relevance score (0.0 to 1.0).
    pub relevance: f64,
}

/// Default maximum number of results to return.
const DEFAULT_LIMIT: usize = 20;

/// Relevance score weights.
const EXACT_NAME_MATCH_SCORE: f64 = 1.0;
const NAME_CONTAINS_SCORE: f64 = 0.8;
const PATH_CONTAINS_SCORE: f64 = 0.5;
const DOCS_CONTAINS_SCORE: f64 = 0.3;

/// Perform a keyword search over the documentation index.
///
/// Supports multiple keywords (space-separated) with AND logic.
/// Results are ranked by relevance and limited to `limit` results.
pub fn search(
    index: &SearchIndex,
    query: &str,
    kind_filter: Option<&ItemKind>,
    limit: Option<usize>,
) -> Vec<SearchResult> {
    let query = query.trim();
    if query.is_empty() {
        return Vec::new();
    }

    let keywords: Vec<String> = query
        .split_whitespace()
        .map(|s| s.to_lowercase())
        .collect();

    let limit = limit.unwrap_or(DEFAULT_LIMIT);

    let mut results: Vec<SearchResult> = index
        .items
        .iter()
        .filter(|item| {
            // Apply kind filter if specified
            if let Some(kind) = kind_filter {
                if &item.kind != kind {
                    return false;
                }
            }
            true
        })
        .filter_map(|item| score_item(item, &keywords))
        .collect();

    // Sort by relevance (descending), then by name (ascending) for stability
    results.sort_by(|a, b| {
        b.relevance
            .partial_cmp(&a.relevance)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.name.cmp(&b.name))
    });

    results.truncate(limit);
    results
}

/// Score a single item against the search keywords.
/// Returns `None` if the item doesn't match all keywords.
fn score_item(item: &DocItem, keywords: &[String]) -> Option<SearchResult> {
    let name_lower = item.name.to_lowercase();
    let path_lower = item.path.to_lowercase();
    let docs_lower = item
        .docs
        .as_ref()
        .map(|d| d.to_lowercase())
        .unwrap_or_default();

    let mut total_score = 0.0;
    let mut all_matched = true;

    for keyword in keywords {
        let mut keyword_score = 0.0;
        let mut matched = false;

        // Exact name match (highest relevance)
        if name_lower == *keyword {
            keyword_score += EXACT_NAME_MATCH_SCORE;
            matched = true;
        }
        // Name contains keyword
        else if name_lower.contains(keyword.as_str()) {
            keyword_score += NAME_CONTAINS_SCORE;
            matched = true;
        }

        // Path contains keyword
        if path_lower.contains(keyword.as_str()) {
            keyword_score += PATH_CONTAINS_SCORE;
            matched = true;
        }

        // Documentation contains keyword
        if docs_lower.contains(keyword.as_str()) {
            keyword_score += DOCS_CONTAINS_SCORE;
            matched = true;
        }

        if !matched {
            all_matched = false;
            break;
        }

        total_score += keyword_score;
    }

    if !all_matched {
        return None;
    }

    // Normalize score to [0.0, 1.0] range
    let max_possible = keywords.len() as f64
        * (EXACT_NAME_MATCH_SCORE + PATH_CONTAINS_SCORE + DOCS_CONTAINS_SCORE);
    let normalized_score = if max_possible > 0.0 {
        (total_score / max_possible).min(1.0)
    } else {
        0.0
    };

    Some(SearchResult {
        name: item.name.clone(),
        kind: item.kind.clone(),
        path: item.path.clone(),
        signature: item.signature.clone(),
        documentation: item.docs.clone(),
        relevance: (normalized_score * 100.0).round() / 100.0, // Round to 2 decimal places
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc_parser::DocItem;

    fn make_test_index() -> SearchIndex {
        SearchIndex {
            items: vec![
                DocItem {
                    id: "1".to_string(),
                    name: "IUnknown".to_string(),
                    kind: ItemKind::Struct,
                    path: "windows::Win32::System::Com::IUnknown".to_string(),
                    docs: Some("Base interface for all COM objects.".to_string()),
                    signature: Some("pub struct IUnknown { ... }".to_string()),
                },
                DocItem {
                    id: "2".to_string(),
                    name: "CoInitializeEx".to_string(),
                    kind: ItemKind::Function,
                    path: "windows::Win32::System::Com::CoInitializeEx".to_string(),
                    docs: Some("Initializes the COM library.".to_string()),
                    signature: Some("fn(dwcoinit: u32) -> ...".to_string()),
                },
                DocItem {
                    id: "3".to_string(),
                    name: "HRESULT".to_string(),
                    kind: ItemKind::Struct,
                    path: "windows::core::HRESULT".to_string(),
                    docs: Some("A 32-bit value used to describe an error or warning.".to_string()),
                    signature: Some("pub struct HRESULT(pub i32)".to_string()),
                },
                DocItem {
                    id: "4".to_string(),
                    name: "Unknown".to_string(),
                    kind: ItemKind::Variant,
                    path: "windows::SomeEnum::Unknown".to_string(),
                    docs: None,
                    signature: None,
                },
            ],
        }
    }

    #[test]
    fn test_search_exact_name() {
        let index = make_test_index();
        let results = search(&index, "IUnknown", None, None);
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "IUnknown");
    }

    #[test]
    fn test_search_case_insensitive() {
        let index = make_test_index();
        let results = search(&index, "iunknown", None, None);
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "IUnknown");
    }

    #[test]
    fn test_search_empty_query() {
        let index = make_test_index();
        let results = search(&index, "", None, None);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_kind_filter() {
        let index = make_test_index();
        let results = search(&index, "IUnknown", Some(&ItemKind::Function), None);
        assert!(results.is_empty());

        let results = search(&index, "IUnknown", Some(&ItemKind::Struct), None);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_limit() {
        let index = make_test_index();
        let results = search(&index, "unknown", None, Some(1));
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_multiple_keywords() {
        let index = make_test_index();
        let results = search(&index, "COM interface", None, None);
        assert!(!results.is_empty());
        // IUnknown has both "COM" and "interface" in its docs
        assert_eq!(results[0].name, "IUnknown");
    }

    #[test]
    fn test_search_no_results() {
        let index = make_test_index();
        let results = search(&index, "nonexistent_xyz", None, None);
        assert!(results.is_empty());
    }
}
