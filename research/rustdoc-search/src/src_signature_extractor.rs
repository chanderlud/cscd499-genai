use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::doc_parser::ItemKind;

/// Extracts item signatures directly from the local windows source tree.
pub struct SourceSignatureExtractor {
    src_root: PathBuf,
}

impl SourceSignatureExtractor {
    /// Create a new extractor pointing at `data/windows/src`.
    pub fn new(src_root: PathBuf) -> Result<Self> {
        let metadata = fs::metadata(&src_root).with_context(|| {
            format!("Failed to access source root {}", src_root.display())
        })?;

        if !metadata.is_dir() {
            anyhow::bail!("Source root is not a directory: {}", src_root.display());
        }

        Ok(Self { src_root })
    }

    /// Extract a source declaration for the given item path.
    pub fn extract_signature(&self, name: &str, kind: &ItemKind, path: &str) -> Option<String> {
        let module_segments = module_segments(path)?;
        let candidate_files = self.candidate_files(&module_segments, name);

        for candidate in candidate_files {
            if let Ok(contents) = fs::read_to_string(&candidate) {
                if let Some(signature) = extract_from_contents(&contents, name, kind) {
                    return Some(signature);
                }
            }
        }

        None
    }

    fn candidate_files(&self, module_segments: &[&str], name: &str) -> Vec<PathBuf> {
        let mut module_dir = self.src_root.join("Windows");
        for segment in module_segments {
            module_dir.push(segment);
        }

        vec![module_dir.join("mod.rs"), module_dir.join(format!("{name}.rs"))]
    }
}

fn module_segments(path: &str) -> Option<Vec<&str>> {
    let stripped = path.strip_prefix("windows::")?;
    let mut segments: Vec<&str> = stripped.split("::").collect();
    if segments.is_empty() {
        return None;
    }

    segments.pop();
    Some(segments)
}

fn extract_from_contents(contents: &str, name: &str, kind: &ItemKind) -> Option<String> {
    let lines: Vec<&str> = contents.lines().collect();

    for (start_idx, line) in lines.iter().enumerate() {
        if declaration_matches(line.trim(), name, kind) {
            return collect_signature(&lines, start_idx, kind);
        }
    }

    None
}

/// If `line` is a public item declaration, returns `Some((ItemKind, name))`; otherwise `None`.
/// Used by the core extractor to discover items when scanning source files.
pub fn parse_public_declaration(line: &str) -> Option<(ItemKind, String)> {
    let line = line.trim();
    if !line.starts_with("pub ") && !line.starts_with("macro_rules! ") {
        return None;
    }

    let name_from_rest = |rest: &str| -> Option<String> {
        let name_end = rest
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .unwrap_or(rest.len());
        let name = rest[..name_end].trim();
        if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        }
    };

    if let Some(rest) = line.strip_prefix("pub struct ") {
        name_from_rest(rest).map(|n| (ItemKind::Struct, n))
    } else if let Some(rest) = line.strip_prefix("pub enum ") {
        name_from_rest(rest).map(|n| (ItemKind::Enum, n))
    } else if let Some(rest) = line.strip_prefix("pub union ") {
        name_from_rest(rest).map(|n| (ItemKind::Union, n))
    } else if let Some(rest) = line.strip_prefix("pub const ") {
        name_from_rest(rest).map(|n| (ItemKind::Constant, n))
    } else if let Some(rest) = line.strip_prefix("pub type ") {
        name_from_rest(rest).map(|n| (ItemKind::TypeAlias, n))
    } else if let Some(rest) = line.strip_prefix("pub trait ") {
        name_from_rest(rest).map(|n| (ItemKind::Trait, n))
    } else if let Some(rest) = line.strip_prefix("pub mod ") {
        name_from_rest(rest).map(|n| (ItemKind::Module, n))
    } else if line.starts_with("macro_rules! ") {
        let rest = line.strip_prefix("macro_rules! ").unwrap();
        name_from_rest(rest).map(|n| (ItemKind::Macro, n))
    } else if line.starts_with("pub macro ") {
        let rest = line.strip_prefix("pub macro ").unwrap();
        name_from_rest(rest).map(|n| (ItemKind::Macro, n))
    } else if line.starts_with("pub fn ") {
        let rest = line.strip_prefix("pub fn ").unwrap();
        name_from_rest(rest).map(|n| (ItemKind::Function, n))
    } else if line.starts_with("pub unsafe fn ") {
        let rest = line.strip_prefix("pub unsafe fn ").unwrap();
        name_from_rest(rest).map(|n| (ItemKind::Function, n))
    } else if line.starts_with("pub async fn ") {
        let rest = line.strip_prefix("pub async fn ").unwrap();
        name_from_rest(rest).map(|n| (ItemKind::Function, n))
    } else if line.starts_with("pub async unsafe fn ") {
        let rest = line.strip_prefix("pub async unsafe fn ").unwrap();
        name_from_rest(rest).map(|n| (ItemKind::Function, n))
    } else if line.starts_with("pub unsafe async fn ") {
        let rest = line.strip_prefix("pub unsafe async fn ").unwrap();
        name_from_rest(rest).map(|n| (ItemKind::Function, n))
    } else if line.starts_with("pub extern ") {
        if let Some(after_fn) = line.split_once("fn ").map(|(_, r)| r) {
            name_from_rest(after_fn).map(|n| (ItemKind::Function, n))
        } else {
            None
        }
    } else {
        None
    }
}

pub fn declaration_matches(line: &str, name: &str, kind: &ItemKind) -> bool {
    match kind {
        ItemKind::Function => {
            matches_named_prefix(line, "pub fn ", name)
                || matches_named_prefix(line, "pub unsafe fn ", name)
                || matches_named_prefix(line, "pub async fn ", name)
                || matches_named_prefix(line, "pub async unsafe fn ", name)
                || matches_named_prefix(line, "pub unsafe async fn ", name)
                || (line.starts_with("pub extern") && contains_named_fn(line, name))
        }
        ItemKind::Struct => matches_named_prefix(line, "pub struct ", name),
        ItemKind::Enum => matches_named_prefix(line, "pub enum ", name),
        ItemKind::Union => matches_named_prefix(line, "pub union ", name),
        ItemKind::Constant => matches_named_prefix(line, "pub const ", name),
        ItemKind::TypeAlias => matches_named_prefix(line, "pub type ", name),
        ItemKind::Trait => matches_named_prefix(line, "pub trait ", name),
        ItemKind::Macro => {
            matches_named_prefix(line, "macro_rules! ", name)
                || matches_named_prefix(line, "pub macro ", name)
        }
        ItemKind::Module => matches_named_prefix(line, "pub mod ", name),
        _ => false,
    }
}

fn matches_named_prefix(line: &str, prefix: &str, name: &str) -> bool {
    if let Some(rest) = line.strip_prefix(prefix) {
        let Some(after_name) = rest.strip_prefix(name) else {
            return false;
        };
        is_name_boundary(after_name.chars().next())
    } else {
        false
    }
}

fn contains_named_fn(line: &str, name: &str) -> bool {
    if let Some((_, after_fn)) = line.split_once("fn ") {
        if let Some(after_name) = after_fn.strip_prefix(name) {
            return is_name_boundary(after_name.chars().next());
        }
    }

    false
}

fn is_name_boundary(next: Option<char>) -> bool {
    matches!(
        next,
        None | Some('<' | '(' | ' ' | '{' | ':' | ';' | '=')
    )
}

pub fn collect_signature(lines: &[&str], start_idx: usize, kind: &ItemKind) -> Option<String> {
    let mut collected_lines = Vec::new();

    for line in lines.iter().skip(start_idx) {
        let trimmed_end = line.trim_end();
        if trimmed_end.trim().is_empty() {
            continue;
        }

        collected_lines.push(trimmed_end);

        let current = collected_lines.join("\n");

        if signature_complete(kind, &current) {
            break;
        }
    }

    let collected = collected_lines.join("\n");
    normalize_signature(kind, &collected)
}

fn signature_complete(kind: &ItemKind, collected: &str) -> bool {
    match kind {
        ItemKind::Function => collected.contains('{') || collected.ends_with(';'),
        ItemKind::Struct | ItemKind::Enum | ItemKind::Union | ItemKind::Trait | ItemKind::Macro => {
            collected.contains('{') || collected.ends_with(';')
        }
        ItemKind::Constant | ItemKind::TypeAlias | ItemKind::Module => collected.ends_with(';'),
        _ => false,
    }
}

pub fn normalize_signature(kind: &ItemKind, collected: &str) -> Option<String> {
    let normalized = collected.trim();
    if normalized.is_empty() {
        return None;
    }

    match kind {
        ItemKind::Function => trim_before_body(normalized),
        ItemKind::Struct | ItemKind::Enum | ItemKind::Union | ItemKind::Trait | ItemKind::Macro => {
            keep_declaration_prefix(normalized)
        }
        ItemKind::Constant | ItemKind::TypeAlias | ItemKind::Module => keep_through_semicolon(normalized),
        _ => None,
    }
}

fn trim_before_body(signature: &str) -> Option<String> {
    if let Some((prefix, _)) = signature.split_once('{') {
        Some(prefix.trim().to_string())
    } else if let Some((prefix, _)) = signature.split_once(';') {
        Some(prefix.trim().to_string())
    } else {
        Some(signature.trim().to_string())
    }
}

fn keep_declaration_prefix(signature: &str) -> Option<String> {
    if let Some((prefix, _)) = signature.split_once('{') {
        Some(format!("{} {{", prefix.trim()))
    } else if let Some((prefix, _)) = signature.split_once(';') {
        Some(format!("{};", prefix.trim()))
    } else {
        Some(signature.trim().to_string())
    }
}

fn keep_through_semicolon(signature: &str) -> Option<String> {
    if let Some((prefix, _)) = signature.split_once(';') {
        Some(format!("{};", prefix.trim()))
    } else {
        Some(signature.trim().to_string())
    }
}
