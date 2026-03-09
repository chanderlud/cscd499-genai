use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::doc_builder::DocSource;

/// Represents the kind of a documentation item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    Module,
    Struct,
    Enum,
    Function,
    Trait,
    Method,
    Constant,
    TypeAlias,
    Macro,
    Variant,
    StructField,
    Union,
    Impl,
    Other,
}

impl ItemKind {
    /// Parse a kind filter string into an `ItemKind`.
    pub fn from_filter(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "module" => Some(Self::Module),
            "struct" => Some(Self::Struct),
            "enum" => Some(Self::Enum),
            "function" | "fn" => Some(Self::Function),
            "trait" => Some(Self::Trait),
            "method" => Some(Self::Method),
            "constant" | "const" => Some(Self::Constant),
            "type" | "typealias" => Some(Self::TypeAlias),
            "macro" => Some(Self::Macro),
            "variant" => Some(Self::Variant),
            "union" => Some(Self::Union),
            _ => None,
        }
    }

}

impl std::fmt::Display for ItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Module => write!(f, "module"),
            Self::Struct => write!(f, "struct"),
            Self::Enum => write!(f, "enum"),
            Self::Function => write!(f, "function"),
            Self::Trait => write!(f, "trait"),
            Self::Method => write!(f, "method"),
            Self::Constant => write!(f, "constant"),
            Self::TypeAlias => write!(f, "typealias"),
            Self::Macro => write!(f, "macro"),
            Self::Variant => write!(f, "variant"),
            Self::StructField => write!(f, "structfield"),
            Self::Union => write!(f, "union"),
            Self::Impl => write!(f, "impl"),
            Self::Other => write!(f, "other"),
        }
    }
}

/// A single documentation item extracted from documentation sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocItem {
    /// Unique identifier.
    pub id: String,
    /// The item's name.
    pub name: String,
    /// The kind of item (struct, enum, function, etc.).
    pub kind: ItemKind,
    /// Full module path (e.g., `windows::Win32::System::Com`).
    pub path: String,
    /// Documentation text (if available).
    pub docs: Option<String>,
    /// Item signature or declaration (if available).
    pub signature: Option<String>,
}

/// A searchable index built from parsed documentation.
#[derive(Debug, Clone)]
pub struct SearchIndex {
    /// All documentation items in a flat list.
    pub items: Vec<DocItem>,
}

impl SearchIndex {
    /// Build from downloaded documentation results.
    pub fn from_docs(_source: &DocSource, paths: &[PathBuf]) -> Result<Self> {
        if let Some(path) = paths.first() {
            Self::from_all_items_html(path)
        } else {
            anyhow::bail!("No HTML file path provided");
        }
    }

    /// Build a search index from the all-items HTML page.
    ///
    /// Parses the `all.html` page from rustdoc output which contains every item
    /// organized under section headers (Structs, Constants, Traits, etc.)
    pub fn from_all_items_html(path: &Path) -> Result<Self> {
        let html = std::fs::read_to_string(path).with_context(|| {
            format!("Failed to read HTML from {}", path.display())
        })?;

        let items = parse_all_items_html(&html)?;
        println!("Parsed {} items from all-items HTML.", items.len());

        Ok(Self { items })
    }

    /// Load supplement items from a JSON file (e.g. `windows-core-items.json`).
    /// Returns an empty vec if the file does not exist.
    pub fn load_supplement(path: &Path) -> Result<Vec<DocItem>> {
        if !path.exists() {
            return Ok(Vec::new());
        }
        let json = std::fs::read_to_string(path).with_context(|| {
            format!("Failed to read supplement from {}", path.display())
        })?;
        let items: Vec<DocItem> = serde_json::from_str(&json).with_context(|| {
            format!("Failed to deserialize supplement from {}", path.display())
        })?;
        Ok(items)
    }

    /// Get the total number of items in the index.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// HTML parser (for downloaded all.html)
// ──────────────────────────────────────────────────────────────────────────────

/// Parse the `all.html` page from rustdoc into `DocItem`s.
///
/// The HTML structure is:
/// ```html
/// <h3 id="structs">Structs</h3>
/// <ul class="all-items">
///   <li><a href="Win32/System/Com/struct.IUnknown.html">Win32::System::Com::IUnknown</a></li>
///   ...
/// </ul>
/// <h3 id="constants">Constants</h3>
/// <ul class="all-items">
///   ...
/// </ul>
/// ```
fn parse_all_items_html(html: &str) -> Result<Vec<DocItem>> {
    let mut items = Vec::new();
    let mut current_kind = ItemKind::Other;
    let mut id_counter: u32 = 0;

    // Simple line-by-line parsing (no HTML parser dependency needed)
    for segment in html.split('<') {
        let segment = segment.trim();

        // Detect section headers: <h3 id="structs">
        if let Some(rest) = segment.strip_prefix("h3 id=\"") {
            if let Some(section_id) = rest.split('"').next() {
                current_kind = section_id_to_kind(section_id);
            }
            continue;
        }

        // Detect item links: after splitting on '<', segments look like:
        //   a href="AI/Actions/struct.ActionEntity.html">AI::Actions::ActionEntity
        // The text content is everything after the '>'
        if let Some(rest) = segment.strip_prefix("a href=\"") {
            if let Some(href_end) = rest.find('"') {
                let href = &rest[..href_end];

                // Skip non-item links (e.g., external links, anchors)
                if !href.ends_with(".html") || href.starts_with("http") {
                    continue;
                }

                // Extract the display text after the closing '>'
                let after_href = &rest[href_end..];
                if let Some(gt_pos) = after_href.find('>') {
                    let display_text = &after_href[gt_pos + 1..];
                    let display_text = display_text.trim();

                    if display_text.is_empty() {
                        continue;
                    }

                    // Determine the item kind from the href or from the current section
                    let kind = kind_from_href(href).unwrap_or(current_kind.clone());

                    // Skip non-useful kinds
                    if matches!(kind, ItemKind::Other | ItemKind::Impl) {
                        continue;
                    }

                    // Build the full path: windows::Path::To::Item
                    let full_path = format!("windows::{}", display_text);

                    // Extract item name (last segment)
                    let name = display_text
                        .rsplit("::")
                        .next()
                        .unwrap_or(display_text)
                        .to_string();

                    id_counter += 1;

                    items.push(DocItem {
                        id: format!("dl-{}", id_counter),
                        name,
                        kind,
                        path: full_path,
                        docs: None,      // HTML listing doesn't include docs
                        signature: None, // HTML listing doesn't include signatures
                    });
                }
            }
        }
    }

    Ok(items)
}

/// Map an HTML section id (e.g., "structs") to our `ItemKind`.
fn section_id_to_kind(id: &str) -> ItemKind {
    match id {
        "structs" => ItemKind::Struct,
        "enums" => ItemKind::Enum,
        "functions" => ItemKind::Function,
        "traits" => ItemKind::Trait,
        "constants" => ItemKind::Constant,
        "types" | "type-aliases" => ItemKind::TypeAlias,
        "macros" => ItemKind::Macro,
        "unions" => ItemKind::Union,
        "modules" => ItemKind::Module,
        "variants" => ItemKind::Variant,
        "methods" => ItemKind::Method,
        _ => ItemKind::Other,
    }
}

/// Try to determine the item kind from the href path.
///
/// Rustdoc URLs follow the pattern: `path/to/struct.Name.html` or `path/to/fn.name.html`.
fn kind_from_href(href: &str) -> Option<ItemKind> {
    // Get the filename part
    let filename = href.rsplit('/').next()?;

    if filename.starts_with("struct.") {
        Some(ItemKind::Struct)
    } else if filename.starts_with("enum.") {
        Some(ItemKind::Enum)
    } else if filename.starts_with("fn.") {
        Some(ItemKind::Function)
    } else if filename.starts_with("trait.") {
        Some(ItemKind::Trait)
    } else if filename.starts_with("constant.") || filename.starts_with("const.") {
        Some(ItemKind::Constant)
    } else if filename.starts_with("type.") {
        Some(ItemKind::TypeAlias)
    } else if filename.starts_with("macro.") {
        Some(ItemKind::Macro)
    } else if filename.starts_with("union.") {
        Some(ItemKind::Union)
    } else if filename.starts_with("mod.") || filename == "index.html" {
        Some(ItemKind::Module)
    } else {
        None
    }
}

