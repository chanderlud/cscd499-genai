//! Standalone binary that extracts `DocItem`s from `data/core/src/` and
//! `data/strings/src/` and writes `data/windows-core-items.json` for the
//! search index supplement.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use rustdoc_search::doc_parser::{DocItem, ItemKind};
use rustdoc_search::src_signature_extractor::{
    collect_signature, declaration_matches, normalize_signature, parse_public_declaration,
};

/// Source roots relative to the project root (current dir when run from rustdoc-search).
const CORE_SRC: &str = "data/core/src";
const STRINGS_SRC: &str = "data/strings/src";
const OUTPUT_PATH: &str = "data/windows-core-items.json";

fn main() -> Result<()> {
    let project_root = std::env::current_dir().context("Failed to get current directory")?;
    let core_root = project_root.join(CORE_SRC);
    let strings_root = project_root.join(STRINGS_SRC);

    let mut items = Vec::new();
    let mut core_count = 0u32;
    let mut str_count = 0u32;

    if core_root.is_dir() {
        let public_modules = public_reexport_modules(&core_root.join("lib.rs"))?;
        for item in walk_and_parse(&core_root, &public_modules, "core", &mut core_count)? {
            items.push(item);
        }
    }

    if strings_root.is_dir() {
        let public_modules = public_reexport_modules(&strings_root.join("lib.rs"))?;
        for item in walk_and_parse(&strings_root, &public_modules, "str", &mut str_count)? {
            items.push(item);
        }
    }

    let out_path = project_root.join(OUTPUT_PATH);
    if let Some(p) = out_path.parent() {
        fs::create_dir_all(p).context("Failed to create output directory")?;
    }
    let file = fs::File::create(&out_path).with_context(|| format!("Failed to create {}", out_path.display()))?;
    serde_json::to_writer_pretty(file, &items).with_context(|| format!("Failed to write {}", out_path.display()))?;

    println!(
        "Wrote {} items (core: {}, strings: {}) to {}.",
        items.len(),
        core_count,
        str_count,
        out_path.display()
    );
    Ok(())
}

/// Parse lib.rs for `pub use module::*` and return the set of module names.
fn public_reexport_modules(lib_path: &Path) -> Result<std::collections::HashSet<String>> {
    let mut set = std::collections::HashSet::new();
    if !lib_path.exists() {
        return Ok(set);
    }
    let content = fs::read_to_string(lib_path).with_context(|| format!("Failed to read {}", lib_path.display()))?;
    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("pub use ") {
            if let Some(module) = rest.split_whitespace().next() {
                if let Some(name) = module.strip_suffix("::*") {
                    set.insert(name.to_string());
                } else if module.contains("::*") {
                    if let Some(name) = module.split("::").next() {
                        set.insert(name.to_string());
                    }
                }
            }
        }
    }
    Ok(set)
}

/// Recursively collect all .rs files under `root`, excluding lib.rs and files under imp/.
/// If `public_modules` is non-empty, only include files whose stem is in that set (for top-level src).
fn collect_rs_files(root: &Path, public_modules: &std::collections::HashSet<String>) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    collect_rs_files_rec(root, root, public_modules, &mut out)?;
    Ok(out)
}

fn collect_rs_files_rec(
    root: &Path,
    dir: &Path,
    public_modules: &std::collections::HashSet<String>,
    out: &mut Vec<PathBuf>,
) -> Result<()> {
    let entries = fs::read_dir(dir).with_context(|| format!("Failed to read dir {}", dir.display()))?;
    for e in entries {
        let e = e.context("Invalid dir entry")?;
        let path = e.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if path.is_dir() {
            if name != "imp" {
                collect_rs_files_rec(root, &path, public_modules, out)?;
            }
            continue;
        }
        if name.ends_with(".rs") && name != "lib.rs" {
            if public_modules.is_empty() {
                out.push(path);
            } else {
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                if public_modules.contains(stem) {
                    out.push(path);
                }
            }
        }
    }
    Ok(())
}

/// Walk a source root and parse all included .rs files into DocItems.
fn walk_and_parse(
    root: &Path,
    public_modules: &std::collections::HashSet<String>,
    id_prefix: &str,
    counter: &mut u32,
) -> Result<Vec<DocItem>> {
    let files = collect_rs_files(root, public_modules)?;
    let mut items = Vec::new();
    for path in files {
        let content = fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
        let lines: Vec<&str> = content.lines().collect();
        let mut doc_buffer: Vec<String> = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("///") {
                let doc_line = trimmed
                    .strip_prefix("///")
                    .map(|s| s.trim_start())
                    .unwrap_or("")
                    .to_string();
                doc_buffer.push(doc_line);
                continue;
            }
            if trimmed.is_empty() {
                continue;
            }
            if let Some((kind, name)) = parse_public_declaration(trimmed) {
                if matches!(kind, ItemKind::Other | ItemKind::Impl) {
                    doc_buffer.clear();
                    continue;
                }
                if !declaration_matches(trimmed, &name, &kind) {
                    doc_buffer.clear();
                    continue;
                }
                let signature = collect_signature(&lines, i, &kind).and_then(|s| normalize_signature(&kind, &s));
                let docs = if doc_buffer.is_empty() {
                    None
                } else {
                    Some(doc_buffer.join("\n"))
                };
                *counter += 1;
                let id = format!("{}-{}", id_prefix, counter);
                let path_str = format!("windows::core::{}", name);
                items.push(DocItem {
                    id,
                    name,
                    kind,
                    path: path_str,
                    docs,
                    signature,
                });
            }
            doc_buffer.clear();
        }
    }
    Ok(items)
}
