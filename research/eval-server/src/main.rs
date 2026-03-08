#[cfg(windows)]
mod windows;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs, io,
    path::Path,
    process::Stdio,
    sync::Arc,
    time::{Duration, Instant},
};
use tempfile::TempDir;
use thiserror::Error;
use tokio::{io::AsyncReadExt, process::Command, sync::Semaphore, time::timeout};
use tower_http::{limit::RequestBodyLimitLayer, trace::TraceLayer};
use tracing::{info, warn};

#[derive(Clone)]
struct AppState {
    semaphore: Arc<Semaphore>,
    api_key: Option<String>,
    limits: RunnerLimits,
}

#[derive(Clone)]
struct RunnerLimits {
    max_output_bytes: usize,
    max_diagnostics: usize,
    build_timeout: Duration,
    clippy_timeout: Duration,
    test_timeout: Duration,
    fmt_timeout: Duration,
}

#[derive(Debug, Error)]
enum AppError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("utf-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("process error: {0}")]
    Process(String),
    #[error("timeout")]
    Timeout,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Timeout => (StatusCode::REQUEST_TIMEOUT, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        (
            status,
            Json(serde_json::json!({ "ok": false, "error": msg })),
        )
            .into_response()
    }
}

#[derive(Deserialize)]
struct EvaluateRequest {
    /// Full contents of src/main.rs
    main_rs: String,
    /// Lines that would live INSIDE the [dependencies] section (no header).
    dependencies: String,
}

#[derive(Serialize)]
struct EvaluateResponse {
    ok: bool,
    project_id: String,
    build: StepReport,
    clippy: StepReport,
    tests: StepReport,
}

#[derive(Deserialize)]
struct FormatRequest {
    /// Rust snippet (file, items, or statements)
    snippet: String,
}

#[derive(Serialize)]
struct FormatResponse {
    ok: bool,
    formatted: Option<String>,
    stderr: String,
    mode: String,
}

#[derive(Serialize, Default, Clone)]
struct StepReport {
    name: String,
    ran: bool,
    ok: bool,
    timed_out: bool,
    exit_code: Option<i32>,
    duration_ms: u128,

    stdout: String,
    stderr: String,
    stdout_truncated: bool,
    stderr_truncated: bool,

    diagnostics: DiagnosticSummary,
    tests: Option<TestSummary>,
}

#[derive(Serialize, Default, Clone)]
struct DiagnosticSummary {
    errors: usize,
    warnings: usize,
    notes: usize,
    helps: usize,
    by_code: BTreeMap<String, usize>,
    items: Vec<DiagItem>,
    truncated: bool,
}

#[derive(Serialize, Clone)]
struct DiagItem {
    level: String,
    code: Option<String>,
    message: String,
    rendered: Option<String>,
}

#[derive(Serialize, Default, Clone)]
struct TestSummary {
    ok: bool,
    passed: u32,
    failed: u32,
    ignored: u32,
    measured: u32,
    filtered_out: u32,
    passed_names: Vec<String>,
    failed_names: Vec<String>,
}

#[derive(Clone)]
struct CmdOut {
    ok: bool,
    timed_out: bool,
    exit_code: Option<i32>,
    duration_ms: u128,
    stdout: String,
    stderr: String,
    stdout_truncated: bool,
    stderr_truncated: bool,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let concurrency: usize = std::env::var("CONCURRENCY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4);

    let api_key = std::env::var("API_KEY").ok();

    let state = AppState {
        semaphore: Arc::new(Semaphore::new(concurrency)),
        api_key,
        limits: RunnerLimits {
            max_output_bytes: 256 * 1024,
            max_diagnostics: 200,
            build_timeout: Duration::from_secs(60),
            clippy_timeout: Duration::from_secs(60),
            test_timeout: Duration::from_secs(60),
            fmt_timeout: Duration::from_secs(10),
        },
    };

    let app = Router::new()
        .route("/evaluate", post(evaluate))
        .route("/format", post(format_snippet))
        .layer(TraceLayer::new_for_http())
        .layer(RequestBodyLimitLayer::new(512 * 1024))
        .with_state(Arc::new(state));

    let addr = "0.0.0.0:3002";
    info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn auth(headers: &HeaderMap, state: &AppState) -> Result<(), AppError> {
    if let Some(expected) = &state.api_key {
        let got = headers
            .get("x-api-key")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if got != expected {
            return Err(AppError::Unauthorized);
        }
    }
    Ok(())
}

async fn evaluate(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<EvaluateRequest>,
) -> Result<Json<EvaluateResponse>, AppError> {
    auth(&headers, &state)?;

    // Keep your VM alive by not letting everyone compile at once.
    let _permit = state.semaphore.acquire().await.unwrap();

    // Minimal guardrail: dependencies should be *entries*, not a whole surprise TOML novel.
    if req.dependencies.contains("\n[") || req.dependencies.trim_start().starts_with('[') {
        return Err(AppError::BadRequest(
            "dependencies must be lines inside [dependencies], not TOML tables".to_string(),
        ));
    }

    let project_id = format!("{:08x}", randish_u32());

    let temp = TempDir::new()?;
    write_project(temp.path(), &req.main_rs, &req.dependencies)?;

    let build = run_cargo_json_step(
        "build",
        temp.path(),
        &["build", "--message-format=json"],
        state.limits.build_timeout,
        &state.limits,
    )
    .await?;

    let mut clippy = StepReport {
        name: "clippy".to_string(),
        ..Default::default()
    };
    let mut tests = StepReport {
        name: "tests".to_string(),
        ..Default::default()
    };

    if build.ok {
        clippy = run_cargo_json_step(
            "clippy",
            temp.path(),
            &["clippy", "--message-format=json"],
            state.limits.clippy_timeout,
            &state.limits,
        )
        .await?;

        tests = run_cargo_test_step(temp.path(), state.limits.test_timeout, &state.limits).await?;
    } else {
        clippy.ran = false;
        tests.ran = false;
    }

    let ok = build.ok && tests.ok; // clippy doesn't “fail” on warnings by default; you still get counts.

    Ok(Json(EvaluateResponse {
        ok,
        project_id,
        build,
        clippy,
        tests,
    }))
}

async fn format_snippet(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<FormatRequest>,
) -> Result<Json<FormatResponse>, AppError> {
    auth(&headers, &state)?;
    let _permit = state.semaphore.acquire().await.unwrap();

    let temp = TempDir::new()?;
    let file_path = temp.path().join("snippet.rs");

    // Try as a standalone Rust file first.
    fs::write(&file_path, &req.snippet)?;

    let attempt1 = run_command_limited(
        "rustfmt",
        &[
            "--edition",
            "2021",
            "--emit",
            "stdout",
            file_path.to_string_lossy().as_ref(),
        ],
        temp.path(),
        state.limits.fmt_timeout,
        state.limits.max_output_bytes,
    )
    .await;

    if let Ok(out) = attempt1 {
        if out.ok {
            return Ok(Json(FormatResponse {
                ok: true,
                formatted: Some(out.stdout),
                stderr: out.stderr,
                mode: "file".to_string(),
            }));
        }
    }

    // If that fails, wrap as statements inside a function and extract it back out.
    let wrapped = format!(
        "fn __snippet_wrapper() {{\n    /*__SNIP_START__*/\n{}\n    /*__SNIP_END__*/\n}}\n",
        req.snippet
    );
    fs::write(&file_path, wrapped)?;

    let out = run_command_limited(
        "rustfmt",
        &[
            "--edition",
            "2021",
            "--emit",
            "stdout",
            file_path.to_string_lossy().as_ref(),
        ],
        temp.path(),
        state.limits.fmt_timeout,
        state.limits.max_output_bytes,
    )
    .await?;

    if !out.ok {
        return Ok(Json(FormatResponse {
            ok: false,
            formatted: None,
            stderr: out.stderr,
            mode: "wrapped".to_string(),
        }));
    }

    let extracted = extract_between_markers(&out.stdout, "/*__SNIP_START__*/", "/*__SNIP_END__*/")
        .unwrap_or_default();
    let dedented = dedent_block(&extracted);

    Ok(Json(FormatResponse {
        ok: true,
        formatted: Some(dedented),
        stderr: out.stderr,
        mode: "wrapped".to_string(),
    }))
}

fn write_project(root: &Path, main_rs: &str, deps_lines: &str) -> Result<(), AppError> {
    let src = root.join("src");
    fs::create_dir_all(&src)?;

    let cargo_toml = format!(
        r#"[package]
name = "submission"
version = "0.1.0"
edition = "2021"

[dependencies]
{}
"#,
        deps_lines.trim()
    );

    let code = if !main_rs.contains("fn main()") {
        format!("fn main() {{}}\n\n{}", main_rs)
    } else {
        main_rs.to_string()
    };

    fs::write(root.join("Cargo.toml"), cargo_toml)?;
    fs::write(src.join("main.rs"), &code)?;
    Ok(())
}

async fn run_cargo_json_step(
    name: &str,
    dir: &Path,
    args: &[&str],
    step_timeout: Duration,
    limits: &RunnerLimits,
) -> Result<StepReport, AppError> {
    let mut report = StepReport {
        name: name.to_string(),
        ran: true,
        ..Default::default()
    };

    let out =
        run_command_limited("cargo", args, dir, step_timeout, limits.max_output_bytes).await?;
    report.ok = out.ok;
    report.timed_out = out.timed_out;
    report.exit_code = out.exit_code;
    report.duration_ms = out.duration_ms;
    report.stdout = out.stdout;
    report.stderr = out.stderr;
    report.stdout_truncated = out.stdout_truncated;
    report.stderr_truncated = out.stderr_truncated;

    // Parse JSON messages (compiler-message etc.)
    report.diagnostics = summarize_cargo_json(&report.stdout, limits.max_diagnostics);
    Ok(report)
}

async fn run_cargo_test_step(
    dir: &Path,
    step_timeout: Duration,
    limits: &RunnerLimits,
) -> Result<StepReport, AppError> {
    let mut report = StepReport {
        name: "tests".to_string(),
        ran: true,
        ..Default::default()
    };

    let out = run_command_limited(
        "cargo",
        &["test"],
        dir,
        step_timeout,
        limits.max_output_bytes,
    )
    .await?;
    report.ok = out.ok;
    report.timed_out = out.timed_out;
    report.exit_code = out.exit_code;
    report.duration_ms = out.duration_ms;
    report.stdout = out.stdout;
    report.stderr = out.stderr;
    report.stdout_truncated = out.stdout_truncated;
    report.stderr_truncated = out.stderr_truncated;

    report.tests = Some(parse_test_summary(&report.stdout, report.ok));
    Ok(report)
}

async fn run_command_limited(
    program: &str,
    args: &[&str],
    dir: &Path,
    step_timeout: Duration,
    max_bytes: usize,
) -> Result<CmdOut, AppError> {
    let start = Instant::now();

    // Windows Job Object that kills the whole process tree when closed/terminated.
    #[cfg(windows)]
    let job = windows::Job::new_kill_on_close().map_err(AppError::Io)?;

    let mut child = Command::new(program)
        .args(args)
        .current_dir(dir)
        .env("CARGO_TERM_COLOR", "never")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| AppError::Process(format!("spawn failed: {e}")))?;

    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::HANDLE;

        let h_process: HANDLE = child.raw_handle().unwrap() as isize;

        // If this fails with ACCESS_DENIED, you're probably already running inside a Job
        // that disallows nesting/breakaway (common in some CI setups).
        job.assign(h_process)
            .map_err(|e| AppError::Process(format!("AssignProcessToJobObject failed: {e}")))?;
    }

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::Process("no stdout".into()))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::Process("no stderr".into()))?;

    let stdout_task = tokio::spawn(async move { read_limited(&mut stdout, max_bytes).await });
    let stderr_task = tokio::spawn(async move { read_limited(&mut stderr, max_bytes).await });

    let status_res = timeout(step_timeout, child.wait()).await;

    let mut timed_out = false;
    let status = match status_res {
        Ok(Ok(s)) => Some(s),
        Ok(Err(e)) => return Err(AppError::Process(format!("wait failed: {e}"))),
        Err(_) => {
            timed_out = true;
            warn!("command timed out: {program} {:?}", args);

            // Kill whole process tree on Windows.
            #[cfg(windows)]
            job.terminate();

            // Also try killing the direct child.
            let _ = child.kill().await;

            // Reap best-effort (don’t hang here forever).
            let _ = timeout(Duration::from_secs(2), child.wait()).await;

            None
        }
    };

    // IMPORTANT: don’t detach IO tasks.
    // If the process tree was killed, pipes should close. If not, don’t hang forever.
    let (stdout_bytes, stdout_trunc) = match timeout(Duration::from_secs(2), stdout_task).await {
        Ok(joined) => joined.map_err(|e| AppError::Process(format!("stdout join: {e}")))??,
        Err(_) => (Vec::new(), true),
    };

    let (stderr_bytes, stderr_trunc) = match timeout(Duration::from_secs(2), stderr_task).await {
        Ok(joined) => joined.map_err(|e| AppError::Process(format!("stderr join: {e}")))??,
        Err(_) => (Vec::new(), true),
    };

    let duration_ms = start.elapsed().as_millis();

    if timed_out {
        return Err(AppError::Timeout);
    }

    let exit_code = status.and_then(|s| s.code());
    let ok = status.map(|s| s.success()).unwrap_or(false);

    Ok(CmdOut {
        ok,
        timed_out,
        exit_code,
        duration_ms,
        stdout: String::from_utf8(stdout_bytes)?,
        stderr: String::from_utf8(stderr_bytes)?,
        stdout_truncated: stdout_trunc,
        stderr_truncated: stderr_trunc,
    })
}

async fn read_limited(
    r: &mut (impl tokio::io::AsyncRead + Unpin),
    limit: usize,
) -> Result<(Vec<u8>, bool), AppError> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 8192];
    let mut truncated = false;

    loop {
        let n = r.read(&mut tmp).await?;
        if n == 0 {
            break;
        }
        if buf.len() < limit {
            let remaining = limit - buf.len();
            let take = remaining.min(n);
            buf.extend_from_slice(&tmp[..take]);
            if take < n {
                truncated = true;
            }
        } else {
            truncated = true;
        }
    }
    Ok((buf, truncated))
}

fn summarize_cargo_json(stdout: &str, max_items: usize) -> DiagnosticSummary {
    let mut sum = DiagnosticSummary::default();

    for line in stdout.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if v.get("reason").and_then(|r| r.as_str()) != Some("compiler-message") {
            continue;
        }
        let Some(msg) = v.get("message") else {
            continue;
        };

        let level = msg
            .get("level")
            .and_then(|x| x.as_str())
            .unwrap_or("unknown")
            .to_string();
        let message = msg
            .get("message")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let rendered = msg
            .get("rendered")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string());
        let code = msg
            .get("code")
            .and_then(|c| c.get("code"))
            .and_then(|x| x.as_str())
            .map(|s| s.to_string());

        match level.as_str() {
            "error" => sum.errors += 1,
            "warning" => sum.warnings += 1,
            "note" => sum.notes += 1,
            "help" => sum.helps += 1,
            _ => {}
        }

        if let Some(c) = &code {
            *sum.by_code.entry(c.clone()).or_insert(0) += 1;
        }

        if sum.items.len() < max_items {
            sum.items.push(DiagItem {
                level,
                code,
                message,
                rendered,
            });
        } else {
            sum.truncated = true;
        }
    }

    sum
}

fn parse_test_summary(stdout: &str, ok: bool) -> TestSummary {
    // Typical line:
    // test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    let re = Regex::new(
        r"test result:\s+(ok|FAILED)\.\s+(\d+)\s+passed;\s+(\d+)\s+failed;\s+(\d+)\s+ignored;\s+(\d+)\s+measured;\s+(\d+)\s+filtered out",
    ).unwrap();
    let test_line_re = Regex::new(r"^test (.+) \.\.\. (ok|FAILED)$").unwrap();

    let mut summary = TestSummary {
        ok,
        ..Default::default()
    };

        for line in stdout.lines() {
            if let Some(caps) = test_line_re.captures(line.trim()) {
                let name = caps[1].to_string();
                match &caps[2] {
                    "ok" => summary.passed_names.push(name),
                    "FAILED" => summary.failed_names.push(name),
                    _ => {}
                }
            }
        }

    if let Some(caps) = re.captures_iter(stdout).last() {
        summary.ok = &caps[1] == "ok";
        summary.passed = caps[2].parse().unwrap_or(0);
        summary.failed = caps[3].parse().unwrap_or(0);
        summary.ignored = caps[4].parse().unwrap_or(0);
        summary.measured = caps[5].parse().unwrap_or(0);
        summary.filtered_out = caps[6].parse().unwrap_or(0);
    }

    summary
}

fn extract_between_markers(s: &str, start: &str, end: &str) -> Option<String> {
    let a = s.find(start)?;
    let b = s[a + start.len()..].find(end)?;
    let inner = &s[a + start.len()..a + start.len() + b];
    Some(inner.to_string())
}

fn dedent_block(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    let min_indent = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.chars().take_while(|c| *c == ' ' || *c == '\t').count())
        .min()
        .unwrap_or(0);

    let mut out = String::new();
    for line in lines {
        let mut cut = line;
        let mut removed = 0usize;
        for (i, ch) in line.char_indices() {
            if removed >= min_indent {
                cut = &line[i..];
                break;
            }
            if ch == ' ' || ch == '\t' {
                removed += 1;
            } else {
                cut = &line[i..];
                break;
            }
        }
        if removed >= min_indent && cut == line {
            // line shorter than indent
            cut = line.trim_start();
        }
        out.push_str(cut);
        out.push('\n');
    }
    out
}

fn randish_u32() -> u32 {
    // Not cryptographic. Just "unique enough" for a temp identifier.
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0));
    (t.as_nanos() as u32) ^ std::process::id().rotate_left(13)
}
