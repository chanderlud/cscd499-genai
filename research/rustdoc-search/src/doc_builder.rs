use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Subdirectory under the project root where generated data files are stored.
const DATA_DIR: &str = "data";

/// Filename for the downloaded all-items HTML page.
const ALL_ITEMS_FILE: &str = "windows-all-items.html";

/// URL for the complete item listing from Microsoft's windows-docs-rs site.
const MICROSOFT_DOCS_ALL_ITEMS_URL: &str =
    "https://microsoft.github.io/windows-docs-rs/doc/windows/all.html";

/// Source of the documentation data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocSource {
    /// Downloaded from the Microsoft docs site (all.html).
    Downloaded,
}

/// Result of documentation acquisition, with source and file paths.
pub struct DocsResult {
    pub source: DocSource,
    pub paths: Vec<PathBuf>,
}

// ──────────────────────────────────────────────────────────────────────────────
// Public API
// ──────────────────────────────────────────────────────────────────────────────

/// Acquire documentation from Microsoft docs.
pub fn acquire_docs(project_root: &Path) -> Result<DocsResult> {
    let data_dir = project_root.join(DATA_DIR);
    fs::create_dir_all(&data_dir).context("Failed to create data directory")?;

    println!("Downloading Windows API documentation from Microsoft docs...");
    let path = download_all_items_page(&data_dir)?;
    println!("Downloaded complete item listing ({}).", format_file_size(&path));
    println!("Source signatures will be loaded from data/windows/src when available.");

    Ok(DocsResult {
        source: DocSource::Downloaded,
        paths: vec![path],
    })
}

/// Check if any documentation data already exists.
pub fn docs_exist(project_root: &Path) -> bool {
    let data_dir = project_root.join(DATA_DIR);
    data_dir.join(ALL_ITEMS_FILE).exists()
}

/// Return existing documentation file paths.
pub fn existing_docs(project_root: &Path) -> Result<DocsResult> {
    let data_dir = project_root.join(DATA_DIR);
    let html_path = data_dir.join(ALL_ITEMS_FILE);
    if html_path.exists() {
        return Ok(DocsResult {
            source: DocSource::Downloaded,
            paths: vec![html_path],
        });
    }

    anyhow::bail!("No documentation files found in {}", data_dir.display());
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
