use anyhow::{Context, Result};
use std::sync::Arc;

mod doc_builder;
mod doc_parser;
mod search;
mod server;
mod src_signature_extractor;

use doc_parser::SearchIndex;
use server::AppState;
use src_signature_extractor::SourceSignatureExtractor;

/// Default server port.
const DEFAULT_PORT: u16 = 3001;

#[tokio::main]
async fn main() -> Result<()> {
    let args = parse_args();

    let project_root = std::env::current_dir().context("Failed to get current directory")?;

    // Step 1: Acquire documentation from Microsoft docs
    let docs_result = if doc_builder::docs_exist(&project_root) {
        doc_builder::existing_docs(&project_root)
            .context("Failed to load existing documentation")?
    } else {
        println!("No downloaded documentation found. Acquiring...");
        doc_builder::acquire_docs(&project_root)
            .context("Failed to acquire documentation")?
    };

    println!(
        "Documentation source: {:?} ({} file(s))",
        docs_result.source,
        docs_result.paths.len()
    );

    // Step 2: Parse documentation into search index
    println!("Loading documentation index...");
    let index = SearchIndex::from_docs(&docs_result.source, &docs_result.paths)
        .context("Failed to parse documentation index")?;

    if index.is_empty() {
        eprintln!("Warning: Documentation index is empty. Search will return no results.");
    } else {
        println!("Indexed {} documentation items.", index.len());
    }

    let src_root = project_root.join("data").join("windows").join("src");
    let sig_extractor = match SourceSignatureExtractor::new(src_root.clone()) {
        Ok(extractor) => {
            println!("Local source signatures enabled from {}.", src_root.display());
            Some(extractor)
        }
        Err(_) => {
            println!(
                "Local source signatures unavailable at {}. Results will omit signatures.",
                src_root.display()
            );
            None
        }
    };

    // Step 3: Start the HTTP server
    let state = Arc::new(AppState {
        index,
        sig_extractor,
    });
    let router = server::build_router(state);

    let port = args.port;
    let addr = format!("127.0.0.1:{}", port);

    println!();
    println!("=== Rustdoc Search Server ===");
    println!("Listening on: http://{}", addr);
    println!();
    println!("Available endpoints:");
    println!("  GET /health           - Health check");
    println!("  GET /search?q=<query> - Search documentation");
    println!("      &kind=<filter>    - Optional: filter by kind (struct, enum, function, trait, etc.)");
    println!("      &limit=<n>        - Optional: limit results (default: 20)");
    println!();
    println!("Example:");
    println!("  curl http://{}/search?q=IUnknown", addr);
    println!("  curl http://{}/search?q=CreateFile&kind=function&limit=5", addr);
    println!();

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("Failed to bind to {}", addr))?;

    // Graceful shutdown on Ctrl+C
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Server error")?;

    println!("Server shut down gracefully.");
    Ok(())
}

/// Wait for a Ctrl+C signal for graceful shutdown.
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install Ctrl+C handler");
    println!("\nReceived Ctrl+C, shutting down...");
}

/// Parsed command-line arguments.
struct Args {
    port: u16,
}

/// Parse command-line arguments manually (no extra dependency needed).
fn parse_args() -> Args {
    let args: Vec<String> = std::env::args().collect();
    let mut port = DEFAULT_PORT;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--port" => {
                i += 1;
                if i < args.len() {
                    port = args[i]
                        .parse()
                        .unwrap_or_else(|_| {
                            eprintln!("Invalid port number: {}. Using default {}.", args[i], DEFAULT_PORT);
                            DEFAULT_PORT
                        });
                } else {
                    eprintln!("--port requires a value. Using default {}.", DEFAULT_PORT);
                }
            }
            other => {
                eprintln!("Unknown argument: {}. Ignoring.", other);
            }
        }
        i += 1;
    }

    Args { port }
}
