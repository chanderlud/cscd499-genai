//! Standalone binary that extracts `DocItem`s from `windows-core` source trees
//! writes `data/windows-core-items.json` for the search index supplement.

use anyhow::{Context, Result};
use rustdoc_search::doc_parser::{DocItem, ItemKind};
use rustdoc_search::src_signature_extractor::{
    collect_signature, normalize_signature, parse_public_declaration,
};
use std::fs;
use std::path::{Path, PathBuf};

const CORE_SRC: &str = "data/core/src";
const STRINGS_SRC: &str = "data/strings/src";
const RESULT_SRC: &str = "data/result/src";
const OUTPUT_PATH: &str = "data/windows-core-items.json";

#[derive(Debug, Clone)]
struct ImplContext {
    impl_type_name: String,
    start_depth: usize,
}

fn main() -> Result<()> {
    let project_root = std::env::current_dir().context("Failed to get current directory")?;
    let crate_roots = [
        (CORE_SRC, "core"),
        (STRINGS_SRC, "str"),
        (RESULT_SRC, "result"),
    ];

    let mut items = Vec::new();
    for (root, id_prefix) in crate_roots {
        let src_root = project_root.join(root);
        if !src_root.is_dir() {
            continue;
        }
        items.extend(walk_and_parse(&src_root, id_prefix)?);
    }

    let out_path = project_root.join(OUTPUT_PATH);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).context("Failed to create output directory")?;
    }
    let file = fs::File::create(&out_path)
        .with_context(|| format!("Failed to create {}", out_path.display()))?;
    serde_json::to_writer_pretty(file, &items)
        .with_context(|| format!("Failed to write {}", out_path.display()))?;

    println!("Wrote {} items to {}.", items.len(), out_path.display());
    Ok(())
}

fn walk_and_parse(root: &Path, id_prefix: &str) -> Result<Vec<DocItem>> {
    let files = collect_rs_files(root)?;
    let mut items = Vec::new();
    let mut counter = 0u32;

    for path in files {
        let content =
            fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
        let lines: Vec<&str> = content.lines().collect();

        let mut doc_buffer: Vec<String> = Vec::new();
        let mut impl_stack: Vec<ImplContext> = Vec::new();
        let mut brace_depth = 0usize;
        let mut in_block_comment = false;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("///") {
                let doc_line = trimmed
                    .strip_prefix("///")
                    .map(|s| s.trim_start())
                    .unwrap_or("")
                    .to_string();
                doc_buffer.push(doc_line);
            } else if !trimmed.is_empty() {
                if brace_depth == 0 {
                    if let Some(impl_type_name) = parse_impl_type_name(&lines, i) {
                        let start_depth = brace_depth + 1;
                        impl_stack.push(ImplContext {
                            impl_type_name,
                            start_depth,
                        });
                    }
                }

                if let Some((parsed_kind, name)) = parse_public_declaration(trimmed) {
                    let docs = if doc_buffer.is_empty() {
                        None
                    } else {
                        Some(doc_buffer.join("\n"))
                    };

                    let mut item_kind = parsed_kind.clone();
                    let mut parent_path = None;
                    let path = if parsed_kind == ItemKind::Function && !impl_stack.is_empty() {
                        item_kind = ItemKind::Method;
                        let parent = format!(
                            "windows::core::{}",
                            impl_stack.last().map(|ctx| ctx.impl_type_name.as_str()).unwrap_or("")
                        );
                        parent_path = Some(parent.clone());
                        format!("{parent}::{name}")
                    } else {
                        format!("windows::core::{name}")
                    };

                    // Method signatures still follow function signature parsing.
                    let signature_kind = if item_kind == ItemKind::Method {
                        ItemKind::Function
                    } else {
                        item_kind.clone()
                    };
                    let signature = collect_signature(&lines, i, &signature_kind)
                        .and_then(|s| normalize_signature(&signature_kind, &s));

                    counter += 1;
                    items.push(DocItem {
                        id: format!("{id_prefix}-{counter}"),
                        name,
                        kind: item_kind,
                        path,
                        docs,
                        signature,
                        parent_path,
                    });
                }

                doc_buffer.clear();
            }

            let cleaned = strip_strings_and_comments(line, &mut in_block_comment);
            let opens = cleaned.chars().filter(|&c| c == '{').count();
            let closes = cleaned.chars().filter(|&c| c == '}').count();
            brace_depth += opens;
            brace_depth = brace_depth.saturating_sub(closes);

            while let Some(ctx) = impl_stack.last() {
                if brace_depth < ctx.start_depth {
                    impl_stack.pop();
                } else {
                    break;
                }
            }
        }
    }

    Ok(items)
}

fn collect_rs_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_rs_files_rec(root, &mut files)?;
    Ok(files)
}

fn collect_rs_files_rec(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let entries = fs::read_dir(dir).with_context(|| format!("Failed to read dir {}", dir.display()))?;
    for entry in entries {
        let entry = entry.context("Invalid dir entry")?;
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files_rec(&path, out)?;
            continue;
        }

        if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
    Ok(())
}

fn parse_impl_type_name(lines: &[&str], start_idx: usize) -> Option<String> {
    let mut header = String::new();
    for line in lines.iter().skip(start_idx) {
        if !header.is_empty() {
            header.push('\n');
        }
        header.push_str(line.trim());

        if find_body_open_brace(&header).is_some() {
            break;
        }
    }

    let header = header.trim();
    if !header.starts_with("impl") {
        return None;
    }

    let idx = find_body_open_brace(header)?;
    let before_body = header[..idx].trim();
    let Some(after_impl) = before_body.strip_prefix("impl") else {
        return None;
    };
    let after_impl = after_impl.trim();
    if after_impl.is_empty() {
        return None;
    }

    // Prefer the implemented target type in trait impls: `impl Trait for Type`.
    let target = if let Some((_, rhs)) = split_top_level_for(after_impl) {
        rhs.trim()
    } else {
        after_impl
    };

    let target = remove_where_clause(target);
    if target.is_empty() {
        None
    } else {
        Some(target.to_string())
    }
}

fn split_top_level_for(s: &str) -> Option<(&str, &str)> {
    let mut angle_depth = 0usize;
    let mut chars = s.char_indices().peekable();

    while let Some((idx, ch)) = chars.next() {
        match ch {
            '<' => angle_depth += 1,
            '>' => angle_depth = angle_depth.saturating_sub(1),
            'f' if angle_depth == 0 => {
                if s[idx..].starts_with("for ") {
                    return Some((&s[..idx], &s[idx + 4..]));
                }
            }
            _ => {}
        }
    }

    None
}

fn remove_where_clause(s: &str) -> &str {
    let mut angle_depth = 0usize;

    for (idx, ch) in s.char_indices() {
        match ch {
            '<' => angle_depth += 1,
            '>' => angle_depth = angle_depth.saturating_sub(1),
            'w' if angle_depth == 0 => {
                if s[idx..].starts_with("where ") {
                    return s[..idx].trim_end();
                }
            }
            _ => {}
        }
    }

    s.trim()
}

fn find_body_open_brace(s: &str) -> Option<usize> {
    let mut angle_depth = 0usize;
    let mut brace_depth = 0usize;

    for (idx, ch) in s.char_indices() {
        match ch {
            '<' => angle_depth += 1,
            '>' => angle_depth = angle_depth.saturating_sub(1),
            '{' if angle_depth == 0 => {
                brace_depth += 1;
                if brace_depth == 1 {
                    return Some(idx);
                }
            }
            '}' if angle_depth == 0 => {
                brace_depth = brace_depth.saturating_sub(1);
            }
            _ => {}
        }
    }

    None
}

fn strip_strings_and_comments(line: &str, in_block_comment: &mut bool) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    let mut in_string = false;
    let mut string_delim = '\0';
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if *in_block_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                *in_block_comment = false;
                let _ = chars.next();
            }
            continue;
        }

        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == string_delim {
                in_string = false;
            }
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'/') {
            break;
        }
        if ch == '/' && chars.peek() == Some(&'*') {
            *in_block_comment = true;
            let _ = chars.next();
            continue;
        }

        if ch == '"' || ch == '\'' {
            in_string = true;
            string_delim = ch;
            escaped = false;
            continue;
        }

        out.push(ch);
    }

    out
}
