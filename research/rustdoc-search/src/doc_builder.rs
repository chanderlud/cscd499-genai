use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Subdirectory under the project root where generated data files are stored.
const DATA_DIR: &str = "data";

/// Filename for the downloaded all-items HTML page.
const ALL_ITEMS_FILE: &str = "windows-all-items.html";

/// Filename prefix for locally generated rustdoc JSON files.
const WINDOWS_JSON_PREFIX: &str = "windows";

/// URL for the complete item listing from Microsoft's windows-docs-rs site.
const MICROSOFT_DOCS_ALL_ITEMS_URL: &str =
    "https://microsoft.github.io/windows-docs-rs/doc/windows/all.html";

/// Source of the documentation data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocSource {
    /// Downloaded from the Microsoft docs site (all.html).
    Downloaded,
    /// Generated locally via cargo doc (rustdoc JSON).
    LocalJson,
}

/// Result of documentation acquisition, with source and file paths.
pub struct DocsResult {
    pub source: DocSource,
    pub paths: Vec<PathBuf>,
}

// ──────────────────────────────────────────────────────────────────────────────
// Public API
// ──────────────────────────────────────────────────────────────────────────────

/// Acquire documentation, preferring download from Microsoft docs.
///
/// Tries to download the `all.html` page first (comprehensive, fast, no nightly needed).
/// Falls back to local generation with `cargo +nightly doc` if the download fails.
pub fn acquire_docs(project_root: &Path) -> Result<DocsResult> {
    let data_dir = project_root.join(DATA_DIR);
    fs::create_dir_all(&data_dir).context("Failed to create data directory")?;

    // Try downloading from Microsoft docs first
    println!("Downloading Windows API documentation from Microsoft docs...");
    match download_all_items_page(&data_dir) {
        Ok(path) => {
            println!("Downloaded complete item listing ({}).", format_file_size(&path));
            Ok(DocsResult {
                source: DocSource::Downloaded,
                paths: vec![path],
            })
        }
        Err(e) => {
            eprintln!("Download failed: {}. Falling back to local generation...", e);
            let paths = generate_docs_locally(project_root)?;
            Ok(DocsResult {
                source: DocSource::LocalJson,
                paths,
            })
        }
    }
}

/// Check if any documentation data already exists.
pub fn docs_exist(project_root: &Path) -> bool {
    let data_dir = project_root.join(DATA_DIR);
    if !data_dir.exists() {
        return false;
    }

    // Check for downloaded HTML
    if data_dir.join(ALL_ITEMS_FILE).exists() {
        return true;
    }

    // Check for locally generated JSON files
    has_json_files(&data_dir)
}

/// Return existing doc file paths, detecting the source type.
pub fn existing_docs(project_root: &Path) -> Result<DocsResult> {
    let data_dir = project_root.join(DATA_DIR);

    // Prefer downloaded HTML
    let html_path = data_dir.join(ALL_ITEMS_FILE);
    if html_path.exists() {
        return Ok(DocsResult {
            source: DocSource::Downloaded,
            paths: vec![html_path],
        });
    }

    // Fall back to JSON files
    let json_paths = collect_existing_json(&data_dir)?;
    if !json_paths.is_empty() {
        return Ok(DocsResult {
            source: DocSource::LocalJson,
            paths: json_paths,
        });
    }

    anyhow::bail!("No documentation files found in {}", data_dir.display());
}

/// Remove all documentation data files (for --rebuild-docs).
pub fn clean_docs(project_root: &Path) -> Result<()> {
    let data_dir = project_root.join(DATA_DIR);
    if !data_dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(&data_dir).context("Failed to read data directory")? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.ends_with(".json") || name.ends_with(".html") {
            fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}

// ──────────────────────────────────────────────────────────────────────────────
// Download from Microsoft docs
// ──────────────────────────────────────────────────────────────────────────────

/// Download the all-items HTML page from Microsoft's docs site.
fn download_all_items_page(data_dir: &Path) -> Result<PathBuf> {
    let output_path = data_dir.join(ALL_ITEMS_FILE);

    println!("  URL: {}", MICROSOFT_DOCS_ALL_ITEMS_URL);

    let response = ureq::get(MICROSOFT_DOCS_ALL_ITEMS_URL)
        .call()
        .context("HTTP request to Microsoft docs failed")?;

    let status = response.status();
    if status != 200 {
        anyhow::bail!("HTTP {} from Microsoft docs", status);
    }

    let body = response
        .into_body()
        .read_to_string()
        .context("Failed to read response body")?;

    if body.len() < 1000 || !body.contains("List of all items") {
        anyhow::bail!(
            "Downloaded page doesn't look like a valid all-items listing (size: {} bytes)",
            body.len()
        );
    }

    fs::write(&output_path, &body).with_context(|| {
        format!("Failed to write to {}", output_path.display())
    })?;

    Ok(output_path)
}

// ──────────────────────────────────────────────────────────────────────────────
// Local generation fallback (cargo doc)
// ──────────────────────────────────────────────────────────────────────────────

/// Generate rustdoc JSON locally using `cargo +nightly doc`.
fn generate_docs_locally(project_root: &Path) -> Result<Vec<PathBuf>> {
    let data_dir = project_root.join(DATA_DIR);
    fs::create_dir_all(&data_dir).context("Failed to create data directory")?;

    let temp_dir = std::env::temp_dir().join("rustdoc-search-temp");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)
            .context("Failed to clean up previous temporary project directory")?;
    }
    fs::create_dir_all(&temp_dir).context("Failed to create temporary project directory")?;

    println!("Creating temporary Cargo project at: {}", temp_dir.display());
    create_temp_project(&temp_dir).context("Failed to create temporary Cargo project")?;

    println!("Generating rustdoc JSON (this may take a while)...");
    run_cargo_doc(&temp_dir).context("Failed to generate rustdoc JSON")?;

    let json_files = copy_json_files(&temp_dir, &data_dir)
        .context("Failed to collect generated JSON files")?;

    if json_files.is_empty() {
        anyhow::bail!("No windows-related JSON files were generated.");
    }

    println!("Saved {} documentation JSON file(s).", json_files.len());

    // Clean up
    if let Err(e) = fs::remove_dir_all(&temp_dir) {
        eprintln!("Warning: Failed to clean up temp directory: {}", e);
    }

    Ok(json_files)
}

/// Create a temporary Cargo project with `windows` as a dependency.
fn create_temp_project(dir: &Path) -> Result<()> {
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).context("Failed to create src directory")?;

    fs::write(src_dir.join("lib.rs"), "// placeholder\n")
        .context("Failed to write lib.rs")?;

    let cargo_toml = r#"[package]
name = "windows-doc-gen"
version = "0.1.0"
edition = "2021"

[dependencies]
windows = { version = "0.62.2", features = [
    "Win32_System_Com",
    "Win32_UI",
    "Win32_UI_Shell",
    "Win32_System_Ole",
    "Win32_System_WindowsProgramming",
    "Win32_System_SystemInformation",
    "Win32_Storage",
    "Win32_Storage_FileSystem",
    "Win32_Security",
] }
"#;

    fs::write(dir.join("Cargo.toml"), cargo_toml).context("Failed to write Cargo.toml")?;

    Ok(())
}

/// Run `cargo +nightly doc` with JSON output.
fn run_cargo_doc(project_dir: &Path) -> Result<()> {
    let output = Command::new("cargo")
        .args(["+nightly", "doc"])
        .current_dir(project_dir)
        .env("RUSTDOCFLAGS", "-Z unstable-options --output-format json")
        .output()
        .context(
            "Failed to execute cargo command. Is the Rust nightly toolchain installed? \
             Install it with: rustup toolchain install nightly",
        )?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("cargo doc failed:\n{}", stderr);
    }

    Ok(())
}

/// Copy windows-related JSON files from the doc output to the data directory.
fn copy_json_files(project_dir: &Path, data_dir: &Path) -> Result<Vec<PathBuf>> {
    let doc_dir = project_dir.join("target").join("doc");
    if !doc_dir.exists() {
        anyhow::bail!("Doc output directory does not exist: {}", doc_dir.display());
    }

    let mut collected = Vec::new();
    for entry in fs::read_dir(&doc_dir).context("Failed to read doc directory")? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with(WINDOWS_JSON_PREFIX) && name_str.ends_with(".json") {
            let dest = data_dir.join(&*name_str);
            fs::copy(entry.path(), &dest)?;
            collected.push(dest);
        }
    }

    collected.sort();
    Ok(collected)
}

// ──────────────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────────────

fn has_json_files(data_dir: &Path) -> bool {
    fs::read_dir(data_dir)
        .ok()
        .map(|entries| {
            entries.filter_map(|e| e.ok()).any(|e| {
                let n = e.file_name();
                let n = n.to_string_lossy();
                n.starts_with(WINDOWS_JSON_PREFIX) && n.ends_with(".json")
            })
        })
        .unwrap_or(false)
}

fn collect_existing_json(data_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = fs::read_dir(data_dir)
        .context("Failed to read data directory")?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let n = e.file_name();
            let n = n.to_string_lossy();
            n.starts_with(WINDOWS_JSON_PREFIX) && n.ends_with(".json")
        })
        .map(|e| e.path())
        .collect();
    paths.sort();
    Ok(paths)
}

fn format_file_size(path: &Path) -> String {
    match fs::metadata(path) {
        Ok(meta) => {
            let bytes = meta.len();
            if bytes >= 1_000_000 {
                format!("{:.1} MB", bytes as f64 / 1_000_000.0)
            } else if bytes >= 1_000 {
                format!("{:.0} KB", bytes as f64 / 1_000.0)
            } else {
                format!("{} bytes", bytes)
            }
        }
        Err(_) => "unknown size".to_string(),
    }
}
