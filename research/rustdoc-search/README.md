# rustdoc-search

A proof-of-concept tool that generates searchable documentation from the `windows` crate and exposes it via an HTTP API with LLM-friendly structured responses.

## Prerequisites

- **Rust stable toolchain** (edition 2024)
- **(Optional) Rust nightly toolchain** — only needed if the download fallback to local generation is triggered
  ```bash
  rustup toolchain install nightly
  ```

## Build

```bash
cd research/rustdoc-search
cargo build
```

## Run

```bash
# First run will download documentation from Microsoft docs (~33 MB)
cargo run

# Force re-download of documentation
cargo run -- --rebuild-docs

# Use a custom port (default: 3000)
cargo run -- --port 8080
```

On the first run, the tool will:
1. Download the complete item listing from [microsoft.github.io/windows-docs-rs](https://microsoft.github.io/windows-docs-rs/doc/windows/all.html) (~33 MB, ~236k items)
2. Parse the HTML into a flat, searchable index
3. Start the HTTP server

If the download fails, it falls back to generating docs locally with `cargo +nightly doc` (slower, partial coverage).

Subsequent runs reuse the cached `data/windows-all-items.html` unless `--rebuild-docs` is passed.

## API Endpoints

### Health Check

```
GET /health
```

Returns the server status and number of indexed items.

**Example:**
```bash
curl http://127.0.0.1:3000/health
```

**Response:**
```json
{
  "status": "ok",
  "indexed_items": 236411
}
```

### Search

```
GET /search?q=<query>[&kind=<filter>][&limit=<n>]
```

Search the documentation index.

**Parameters:**
| Parameter | Required | Description |
|-----------|----------|-------------|
| `q`       | Yes      | Search query (space-separated keywords, AND logic) |
| `kind`    | No       | Filter by item kind: `module`, `struct`, `enum`, `function`, `trait`, `method`, `constant`, `type`, `macro`, `variant`, `union` |
| `limit`   | No       | Maximum number of results (default: 20) |

**Example queries:**
```bash
# Search for HWND
curl "http://127.0.0.1:3000/search?q=HWND"

# Search for COM functions only, limit to 5 results
curl "http://127.0.0.1:3000/search?q=CoCreateInstance&kind=function&limit=5"

# Search for file system functions
curl "http://127.0.0.1:3000/search?q=CreateFile&kind=function"

# Multi-keyword search
curl "http://127.0.0.1:3000/search?q=COM+interface"
```

**Response:**
```json
{
  "query": "CoCreateInstance",
  "total_results": 5,
  "results": [
    {
      "name": "CoCreateInstance",
      "kind": "function",
      "path": "windows::Win32::System::Com::CoCreateInstance",
      "signature": null,
      "documentation": null,
      "relevance": 0.83
    }
  ]
}
```

## Data Sources

The tool supports two documentation sources:

1. **Download from Microsoft docs** (preferred) — Downloads the complete all-items listing from the [windows-docs-rs](https://microsoft.github.io/windows-docs-rs/doc/windows/) site. This provides comprehensive coverage of the entire Windows API (~236k items) without requiring a nightly Rust toolchain.

2. **Local generation** (fallback) — Creates a temporary project and runs `cargo +nightly doc --output-format json` to generate rustdoc JSON. Requires the nightly toolchain, works offline, but only covers features explicitly enabled (~2k items).

## Architecture

The project is organized into four modules:

- **`doc_builder`** - Acquires documentation (download from Microsoft docs, or local generation fallback)
- **`doc_parser`** - Parses documentation into a flat searchable index (supports both HTML and rustdoc JSON formats)
- **`search`** - Keyword search with relevance ranking (exact match > name contains > path contains > docs contains)
- **`server`** - Axum HTTP server with CORS support

## Testing

```bash
cargo test
```
