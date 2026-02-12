//! # Rustdoc Search
//!
//! A tool that generates searchable documentation from the `windows` crate
//! and exposes it via an HTTP API with LLM-friendly structured responses.
//!
//! ## Data Sources
//!
//! The tool supports two documentation sources:
//!
//! 1. **Download from Microsoft docs** (preferred) — Downloads the complete all-items listing
//!    from <https://microsoft.github.io/windows-docs-rs/doc/windows/all.html>.
//!    This provides comprehensive coverage of the entire Windows API without requiring
//!    a nightly Rust toolchain.
//!
//! 2. **Local generation** (fallback) — Creates a temporary project and runs
//!    `cargo +nightly doc --output-format json` to generate rustdoc JSON.
//!    Requires the nightly toolchain but works offline.
//!
//! ## Modules
//!
//! - [`doc_builder`] - Acquires documentation (download or local generation).
//! - [`doc_parser`] - Parses documentation into a flat, searchable index.
//! - [`search`] - Keyword-based search with relevance scoring.
//! - [`server`] - HTTP API (Axum) for searching the documentation.
//!
//! ## Search Behavior
//!
//! - Queries are split into space-separated keywords with AND logic.
//! - Matching is case-insensitive against item names, module paths, and documentation text.
//! - Relevance ranking: exact name match (1.0) > name contains (0.8) > path contains (0.5) > docs contains (0.3).
//! - Results can be filtered by item kind and limited in count.

pub mod doc_builder;
pub mod doc_parser;
pub mod search;
pub mod server;
