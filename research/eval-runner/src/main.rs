use anyhow::{anyhow, Context, Result};
use clap::Parser;
use futures::stream::{self, StreamExt};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use std::fs::{File};
use std::time::Duration;
use tokio::time::sleep;

const WINDOWS_DEPENDENCIES: &str = r#"windows = { version = "0.62.2", features = ["Win32_System_Com", "Win32_UI", "Win32_UI_Shell", "Win32_System_Ole", "Win32_System_WindowsProgramming", "Win32_System_SystemInformation", "Win32_Storage", "Win32_Storage_FileSystem", "Win32_Security"] }"#;
const CARGO_DEPENDENCIES: &str = r#"regex = "*"
rand = "*"
md5 = "*"
"#;
const MAX_RETRIES: usize = 10;

#[derive(Parser, Debug, Clone)]
#[command(name = "rust-eval-runner")]
struct Cli {
    /// Prompt template file. Use {{problem}} where the problem text should be inserted.
    #[arg(long)]
    template: PathBuf,

    /// Problems file: .json (array of strings) OR plaintext (one problem per line).
    #[arg(long)]
    problems: PathBuf,

    /// Attempts per problem
    #[arg(long, default_value_t = 3)]
    k: usize,

    /// OpenRouter model id (e.g. "openai/gpt-4.1-mini", "anthropic/claude-3.5-sonnet")
    #[arg(long, default_value = "openai/gpt-4o-mini")]
    model: String,

    /// OpenRouter API key (or set OPENROUTER_API_KEY)
    #[arg(long, env = "OPENROUTER_API_KEY")]
    openrouter_key: String,

    /// Optional attribution: set OPENROUTER_HTTP_REFERER
    #[arg(long, env = "OPENROUTER_HTTP_REFERER")]
    openrouter_http_referer: Option<String>,

    /// Optional attribution: set OPENROUTER_X_TITLE
    #[arg(long, env = "OPENROUTER_X_TITLE")]
    openrouter_x_title: Option<String>,

    /// OpenRouter base URL
    #[arg(long, default_value = "https://openrouter.ai/api/v1")]
    openrouter_base: String,

    /// Evaluation API base URL (your service)
    #[arg(long, default_value = "http://localhost:3000")]
    eval_base: String,

    /// Optional X-API-Key header for your evaluation API
    #[arg(long, env = "EVAL_API_KEY")]
    eval_api_key: Option<String>,

    /// Temperature for generation
    #[arg(long, default_value_t = 0.2)]
    temperature: f32,

    /// Max tokens for generation
    #[arg(long, default_value_t = 1400)]
    max_tokens: u32,

    /// Concurrency of attempts (LLM + eval) within a problem
    #[arg(long, default_value_t = 2)]
    concurrency: usize,

    /// Output directory (writes run_report.json + summary.csv)
    #[arg(long, default_value = "./out")]
    out_dir: PathBuf,

    /// Save full attempt artifacts (prompts, raw model text, full eval stdout/stderr) in run_report.json
    #[arg(long, default_value_t = false)]
    save_artifacts: bool,
}

#[derive(Serialize)]
struct RunReport {
    meta: RunMeta,
    problems: Vec<ProblemReport>,
}

#[derive(Serialize)]
struct RunMeta {
    started_unix_ms: u128,
    model: String,
    k: usize,
    eval_base: String,
    openrouter_base: String,
}

#[derive(Serialize)]
struct ProblemReport {
    problem_id: String,
    problem: String,
    attempts: Vec<AttemptReport>,
    stats: ProblemStats,
}

#[derive(Serialize)]
struct AttemptReport {
    attempt: usize,
    prompt: Option<String>,
    raw_model_text: Option<String>,
    generation_error: Option<String>,
    eval: Option<EvaluateResponse>,
    eval_error: Option<String>,
}

/// Your evaluation API request
#[derive(Serialize)]
struct EvaluateRequest {
    main_rs: String,
    dependencies: String,
}

/// Your evaluation API response (matches what you described earlier)
#[derive(Serialize, Deserialize, Clone)]
struct EvaluateResponse {
    ok: bool,
    project_id: String,
    build: StepReport,
    clippy: StepReport,
    tests: StepReport,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct StepReport {
    name: String,
    ran: bool,
    ok: bool,
    timed_out: bool,
    exit_code: Option<i32>,
    duration_ms: u64,

    stdout: String,
    stderr: String,
    stdout_truncated: bool,
    stderr_truncated: bool,

    diagnostics: DiagnosticSummary,
    tests: Option<TestSummary>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct DiagnosticSummary {
    errors: usize,
    warnings: usize,
    notes: usize,
    helps: usize,
    by_code: BTreeMap<String, usize>,
    items: Vec<DiagItem>,
    truncated: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct DiagItem {
    level: String,
    code: Option<String>,
    message: String,
    rendered: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct TestSummary {
    ok: bool,
    passed: u32,
    failed: u32,
    ignored: u32,
    measured: u32,
    filtered_out: u32,
}

/// OpenRouter chat completion response (OpenAI-compatible)
#[derive(Deserialize)]
struct OrChatResponse {
    choices: Vec<OrChoice>,
    usage: Option<OrUsage>,
    model: Option<String>,
    id: Option<String>,
}

#[derive(Deserialize)]
struct OrChoice {
    message: OrMessage,
    finish_reason: Option<String>,
    index: Option<u32>,
}

#[derive(Deserialize)]
struct OrMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OrUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    total_tokens: Option<u32>,
}

#[derive(Serialize)]
struct ProblemStats {
    attempts: usize,
    build_success_rate: f64,
    test_success_rate: f64,
    overall_ok_rate: f64,

    avg_clippy_warnings: f64,
    avg_clippy_errors: f64,

    top_build_codes: Vec<(String, u64)>,
    top_clippy_codes: Vec<(String, u64)>,
}

#[derive(Deserialize)]
struct ProblemContents {
    prompt: String,
    tests: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    fs::create_dir_all(&cli.out_dir).context("create out_dir")?;

    let template = fs::read_to_string(&cli.template).context("read template")?;
    let problems = load_problems(&cli.problems)?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .context("build reqwest client")?;

    let started_unix_ms = now_unix_ms();

    let problem_count = problems.len();
    let mut problem_reports = Vec::with_capacity(problem_count);

    for (index, (problem_id, problem)) in problems.into_iter().enumerate() {
        let attempts = run_problem(
            &client,
            &cli,
            &template,
            &problem_id,
            &problem,
        )
            .await?;

        println!("[{}/{problem_count}] evaluated problem {problem_id}", index + 1);

        let stats = compute_stats(&attempts);

        problem_reports.push(ProblemReport {
            problem_id,
            problem: problem.prompt,
            attempts,
            stats,
        });
    }

    let report = RunReport {
        meta: RunMeta {
            started_unix_ms,
            model: cli.model.clone(),
            k: cli.k,
            eval_base: cli.eval_base.clone(),
            openrouter_base: cli.openrouter_base.clone(),
        },
        problems: problem_reports,
    };

    // Write JSON report
    let json_path = cli.out_dir.join("run_report.json");
    fs::write(&json_path, serde_json::to_vec_pretty(&report)?)
        .context("write run_report.json")?;

    // Write CSV summary (one row per problem)
    let csv_path = cli.out_dir.join("summary.csv");
    write_summary_csv(&csv_path, &report)?;

    eprintln!("Wrote:");
    eprintln!("  {}", json_path.display());
    eprintln!("  {}", csv_path.display());

    Ok(())
}

async fn run_problem(
    client: &reqwest::Client,
    cli: &Cli,
    template: &str,
    problem_id: &str,
    problem: &ProblemContents,
) -> Result<Vec<AttemptReport>> {
    let prompt = template.replace("{{problem}}", &problem.prompt);

    let attempt_indices: Vec<usize> = (1..=cli.k).collect();

    let mut attempts: Vec<AttemptReport> = stream::iter(attempt_indices)
        .map(|attempt| {
            let client = client.clone();
            let cli = cli.clone();
            let prompt = prompt.clone();
            let problem_id = problem_id.to_string();
            let tests = problem.tests.clone();
            async move {
                run_attempt(&client, &cli, &problem_id, attempt, &prompt, &tests).await
            }
        })
        .buffer_unordered(cli.concurrency)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?;

    attempts.sort_by_key(|a| a.attempt);
    Ok(attempts)
}

async fn run_attempt(
    client: &reqwest::Client,
    cli: &Cli,
    _problem_id: &str,
    attempt: usize,
    prompt: &str,
    tests: &str,
) -> Result<AttemptReport> {
    let mut retries = 0;
    let mut out = AttemptReport {
        attempt,
        prompt: cli.save_artifacts.then(|| prompt.to_string()),
        raw_model_text: None,
        generation_error: None,
        eval: None,
        eval_error: None,
    };

    let output = loop {
        // 1) Generate with OpenRouter
        let raw_text = match openrouter_chat(client, cli, prompt).await {
            Ok(t) => t,
            Err(e) => {
                if retries > MAX_RETRIES {
                    out.generation_error = Some(e.to_string());
                    return Ok(out);
                } else {
                    retries += 1;
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
            }
        };

        // 2) strip common formatting issues
        let stripped_a = raw_text.trim_ascii();
        let stripped_b = stripped_a.strip_prefix("```rust\n").unwrap_or(stripped_a);
        let stripped_c = stripped_b.strip_suffix("\n```").unwrap_or(stripped_b);

        if cli.save_artifacts {
            out.raw_model_text = Some(stripped_c.to_string());
        }

        if stripped_c.is_empty() {
            if retries > MAX_RETRIES {
                out.eval_error = Some("empty model output".to_string());
                return Ok(out);
            } else {
                retries += 1;
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        } else {
            break stripped_c.to_string();
        }
    };

    // 3) Join the model output with the prepared unit tests
    let eval_text = format!("{}\n\n{}", output, tests);

    // 4) Evaluate
    match eval_code(client, cli, &eval_text).await {
        Ok(mut resp) => {
            if !cli.save_artifacts {
                // Strip big fields unless requested
                strip_step_io(&mut resp.build);
                strip_step_io(&mut resp.clippy);
                strip_step_io(&mut resp.tests);
            }
            out.eval = Some(resp);
        }
        Err(e) => {
            out.eval_error = Some(e.to_string());
        }
    }

    Ok(out)
}

fn strip_step_io(step: &mut StepReport) {
    step.stdout.clear();
    step.stderr.clear();
    step.diagnostics.items.truncate(10);
}

async fn openrouter_chat(client: &reqwest::Client, cli: &Cli, prompt: &str) -> Result<String> {
    let url = format!("{}/chat/completions", cli.openrouter_base.trim_end_matches('/'));

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", cli.openrouter_key))
            .context("bad OPENROUTER_API_KEY")?,
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    // Optional attribution headers. OpenRouter documents these as optional
    if let Some(r) = &cli.openrouter_http_referer {
        headers.insert("HTTP-Referer", HeaderValue::from_str(r)?);
    }
    if let Some(t) = &cli.openrouter_x_title {
        headers.insert("X-Title", HeaderValue::from_str(t)?);
    }

    // OpenRouter is OpenAI-compatible: messages + model + sampling params
    let body = json!({
        "model": cli.model,
        "messages": [
            { "role": "user", "content": prompt }
        ],
        "temperature": cli.temperature,
        "max_tokens": cli.max_tokens
    });

    let resp = client
        .post(url)
        .headers(headers)
        .json(&body)
        .send()
        .await
        .context("OpenRouter request failed")?;

    let status = resp.status();
    let text = resp.text().await.context("read OpenRouter response")?;

    if !status.is_success() {
        return Err(anyhow!("OpenRouter HTTP {}: {}", status, text));
    }

    let parsed: OrChatResponse =
        serde_json::from_str(&text).context("parse OpenRouter JSON")?;

    let content = parsed
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| anyhow!("OpenRouter response had no choices[0].message.content"))?;

    Ok(content)
}

async fn eval_code(client: &reqwest::Client, cli: &Cli, gen: &str) -> Result<EvaluateResponse> {
    let url = format!("{}/evaluate", cli.eval_base.trim_end_matches('/'));
    let mut req = client.post(url).header(CONTENT_TYPE, "application/json");

    if let Some(k) = &cli.eval_api_key {
        req = req.header("X-API-Key", k);
    }

    let body = EvaluateRequest {
        main_rs: gen.to_string(),
        dependencies: CARGO_DEPENDENCIES.to_string(),
    };

    let resp = req
        .json(&body)
        .send().await.context("eval API request failed")?;
    let status = resp.status();
    let text = resp.text().await.context("read eval API response")?;

    if !status.is_success() {
        return Err(anyhow!("eval API HTTP {}: {}", status, text));
    }

    let parsed: EvaluateResponse =
        serde_json::from_str(&text).context("parse eval API JSON")?;
    Ok(parsed)
}

fn compute_stats(attempts: &[AttemptReport]) -> ProblemStats {
    let mut build_ok = 0usize;
    let mut test_ok = 0usize;
    let mut overall_ok = 0usize;

    let mut clippy_warn_sum: u64 = 0;
    let mut clippy_err_sum: u64 = 0;

    let mut build_codes: HashMap<String, u64> = HashMap::new();
    let mut clippy_codes: HashMap<String, u64> = HashMap::new();

    let attempts_n = attempts.len();

    for a in attempts {
        if let Some(ev) = &a.eval {
            if ev.build.ok {
                build_ok += 1;
            }
            if ev.tests.ok {
                test_ok += 1;
            }
            if ev.ok {
                overall_ok += 1;
            }

            clippy_warn_sum += ev.clippy.diagnostics.warnings as u64;
            clippy_err_sum += ev.clippy.diagnostics.errors as u64;

            for (code, ct) in &ev.build.diagnostics.by_code {
                *build_codes.entry(code.clone()).or_insert(0) += *ct as u64;
            }
            for (code, ct) in &ev.clippy.diagnostics.by_code {
                *clippy_codes.entry(code.clone()).or_insert(0) += *ct as u64;
            }
        }
    }

    let denom = attempts_n.max(1) as f64;

    ProblemStats {
        attempts: attempts_n,
        build_success_rate: build_ok as f64 / denom,
        test_success_rate: test_ok as f64 / denom,
        overall_ok_rate: overall_ok as f64 / denom,
        avg_clippy_warnings: clippy_warn_sum as f64 / denom,
        avg_clippy_errors: clippy_err_sum as f64 / denom,
        top_build_codes: top_n(build_codes, 10),
        top_clippy_codes: top_n(clippy_codes, 10),
    }
}

fn top_n(map: HashMap<String, u64>, n: usize) -> Vec<(String, u64)> {
    let mut v: Vec<(String, u64)> = map.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1));
    v.truncate(n);
    v
}

fn write_summary_csv(path: &Path, report: &RunReport) -> Result<()> {
    let mut wtr = csv::Writer::from_path(path).context("create summary.csv")?;

    wtr.write_record([
        "problem_id",
        "attempts",
        "build_success_rate",
        "test_success_rate",
        "overall_ok_rate",
        "avg_clippy_warnings",
        "avg_clippy_errors",
    ])?;

    for p in &report.problems {
        wtr.write_record([
            p.problem_id.as_str(),
            &p.stats.attempts.to_string(),
            &format!("{:.4}", p.stats.build_success_rate),
            &format!("{:.4}", p.stats.test_success_rate),
            &format!("{:.4}", p.stats.overall_ok_rate),
            &format!("{:.4}", p.stats.avg_clippy_warnings),
            &format!("{:.4}", p.stats.avg_clippy_errors),
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

fn load_problems(path: &Path) -> Result<HashMap<String, ProblemContents>> {
    let mut problem_file = File::open(path)?;
    let problems = serde_json::from_reader(&mut problem_file)?;
    Ok(problems)
}

fn now_unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}