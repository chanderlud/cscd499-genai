use anyhow::{anyhow, Context, Result};
use clap::{Parser, ValueEnum};
use futures::stream::{self, StreamExt};
use regex::Regex;
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

const CARGO_DEPENDENCIES: &str = include_str!("../../rust_dependencies.md");
const MAX_RETRIES: usize = 10;
const MAX_EMPTY_RETRIES: usize = 5;

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
    #[arg(long, default_value = "qwen2.5-coder:latest")]
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
    #[arg(long, default_value = "http://localhost:11434/v1")]
    openrouter_base: String,

    /// Evaluation API base URL (your service)
    #[arg(long, default_value = "http://localhost:3002")]
    eval_base: String,

    /// Optional X-API-Key header for your evaluation API
    #[arg(long, env = "EVAL_API_KEY")]
    eval_api_key: Option<String>,

    /// Temperature for generation
    #[arg(long, default_value_t = 0.2)]
    temperature: f32,

    /// Max tokens for generation
    #[arg(long, default_value_t = 16000)]
    max_tokens: u32,

    /// Concurrency of problems
    #[arg(long, default_value_t = 5)]
    concurrency: usize,

    /// Output directory (writes run_report.json + summary.csv)
    #[arg(long, default_value = "./out")]
    out_dir: PathBuf,

    /// Save full attempt artifacts (prompts, raw model text, full eval stdout/stderr) in run_report.json
    #[arg(long, default_value_t = true)]
    save_artifacts: bool,

    /// Prompt strategy mode for template handling.
    #[arg(long, value_enum, default_value = "system-user")]
    prompt_mode: PromptMode,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum PromptMode {
    Raw,
    SystemUser,
    FewShot,
}

#[derive(Clone, Debug)]
struct PromptTemplate {
    system: Option<String>,
    user: String,
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
    retries: usize,
    finish_reason: Option<String>,
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
    content: Option<String>,
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

    let template_raw = fs::read_to_string(&cli.template).context("read template")?;
    let template = parse_prompt_template(&template_raw, cli.prompt_mode);
    let problems = load_problems(&cli.problems)?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(240))
        .build()
        .context("build reqwest client")?;

    warmup_eval_cache(&client, &cli).await;

    let started_unix_ms = now_unix_ms();

    let problem_count = problems.len();

    let problem_reports: Vec<_> = stream::iter(problems.into_iter().enumerate())
        .map(|(index, (problem_id, problem))| {
            let client = client.clone();
            let cli = cli.clone();
            let template = template.clone();

            async move {
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

                Ok::<ProblemReport, anyhow::Error>(ProblemReport {
                    problem_id,
                    problem: problem.prompt,
                    attempts,
                    stats,
                })
            }
        })
        .buffer_unordered(cli.concurrency)
        .collect()
        .await;

    let report = RunReport {
        meta: RunMeta {
            started_unix_ms,
            model: cli.model.clone(),
            k: cli.k,
            eval_base: cli.eval_base.clone(),
            openrouter_base: cli.openrouter_base.clone(),
        },
        problems: problem_reports.into_iter().filter_map(|r| r.ok()).collect(),
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

async fn warmup_eval_cache(client: &reqwest::Client, cli: &Cli) {
    let url = format!("{}/warmup", cli.eval_base.trim_end_matches('/'));
    let mut req = client.get(url);
    if let Some(k) = &cli.eval_api_key {
        req = req.header("X-API-Key", k);
    }

    match req.send().await {
        Ok(resp) => {
            let status = resp.status();
            match resp.text().await {
                Ok(body) => {
                    if status.is_success() {
                        eprintln!("eval warmup succeeded: {body}");
                    } else {
                        eprintln!("eval warmup failed HTTP {}: {}", status, body);
                    }
                }
                Err(err) => eprintln!("eval warmup body read failed: {err}"),
            }
        }
        Err(err) => eprintln!("eval warmup request failed: {err}"),
    }
}

async fn run_problem(
    client: &reqwest::Client,
    cli: &Cli,
    template: &PromptTemplate,
    problem_id: &str,
    problem: &ProblemContents,
) -> Result<Vec<AttemptReport>> {
    let user_prompt = template.user.replace("{{problem}}", &problem.prompt);
    let system_prompt = template.system.clone();
    let mut attempts = Vec::with_capacity(cli.k);

    for attempt in 1..=cli.k {
        match run_attempt(
            &client,
            &cli,
            &problem_id,
            attempt,
            system_prompt.as_deref(),
            &user_prompt,
            &problem.tests,
        )
        .await
        {
            Ok(result) => attempts.push(result),
            Err(error) => eprintln!("attempt {attempt} failed for {problem_id}: {error}")
        }
    }

    attempts.sort_by_key(|a| a.attempt);
    Ok(attempts)
}

async fn run_attempt(
    client: &reqwest::Client,
    cli: &Cli,
    _problem_id: &str,
    attempt: usize,
    system_prompt: Option<&str>,
    user_prompt: &str,
    tests: &str,
) -> Result<AttemptReport> {
    let mut api_retries = 0usize;
    let mut empty_retries = 0usize;
    let mut last_finish_reason: Option<String> = None;
    let mut prompt_suffix = String::new();
    let mut length_retry_boost_pending = false;
    let mut used_length_retry_boost = false;
    let mut stop_empty_retries = 0usize;
    let mut out = AttemptReport {
        attempt,
        prompt: cli.save_artifacts.then(|| user_prompt.to_string()),
        raw_model_text: None,
        generation_error: None,
        eval: None,
        eval_error: None,
        retries: 0,
        finish_reason: None,
    };

    let output = loop {
        // 1) Generate with OpenRouter
        let request_prompt = format!("{user_prompt}{prompt_suffix}");
        let temp_override = Some((cli.temperature + (empty_retries as f32 * 0.05)).min(1.0));
        let max_tokens_override = if length_retry_boost_pending && !used_length_retry_boost {
            used_length_retry_boost = true;
            length_retry_boost_pending = false;
            Some(cli.max_tokens.saturating_mul(2))
        } else {
            None
        };

        let (raw_text, finish_reason) = match openrouter_chat(
            client,
            cli,
            system_prompt,
            &request_prompt,
            temp_override,
            max_tokens_override,
        )
        .await
        {
            Ok(result) => result,
            Err(e) => {
                println!("{}", e);
                if api_retries >= MAX_RETRIES {
                    out.generation_error = Some(e.to_string());
                    out.retries = api_retries + empty_retries;
                    out.finish_reason = last_finish_reason;
                    return Ok(out);
                } else {
                    api_retries += 1;
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
            }
        };

        last_finish_reason = finish_reason.clone();
        if cli.save_artifacts {
            out.raw_model_text = Some(raw_text.clone());
        }

        let extracted = extract_rust_code(&raw_text);

        if extracted.is_none() {
            if finish_reason.as_deref() == Some("content_filter") {
                eprintln!("model output blocked by content filter; not retrying this attempt");
                out.eval_error = Some("blocked by content filter".to_string());
                out.retries = api_retries + empty_retries;
                out.finish_reason = last_finish_reason;
                return Ok(out);
            }

            if finish_reason.as_deref() == Some("length") {
                eprintln!("model output ended due to token length with empty content");
                length_retry_boost_pending = true;
                stop_empty_retries = 0;
            } else if finish_reason.as_deref() == Some("stop") {
                stop_empty_retries += 1;
                if stop_empty_retries >= 3 {
                    prompt_suffix =
                        "\n\nPlease provide the complete Rust implementation.".to_string();
                }
            } else {
                stop_empty_retries = 0;
            }

            if empty_retries >= MAX_EMPTY_RETRIES {
                out.eval_error = Some("empty model output".to_string());
                out.retries = api_retries + empty_retries;
                out.finish_reason = last_finish_reason;
                return Ok(out);
            } else {
                empty_retries += 1;
                let sleep_secs = (1u64 << (empty_retries - 1)).min(30);
                sleep(Duration::from_secs(sleep_secs)).await;
                continue;
            }
        } else {
            break extracted.unwrap_or_default();
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

    out.retries = api_retries + empty_retries;
    out.finish_reason = last_finish_reason;
    Ok(out)
}

fn strip_step_io(step: &mut StepReport) {
    step.stdout.clear();
    step.stderr.clear();
    step.diagnostics.items.truncate(10);
}

fn parse_prompt_template(template: &str, mode: PromptMode) -> PromptTemplate {
    if matches!(mode, PromptMode::Raw) {
        return PromptTemplate {
            system: None,
            user: template.to_string(),
        };
    }

    if let Some((system, user)) = split_system_user_template(template) {
        return PromptTemplate {
            system: Some(system),
            user,
        };
    }

    PromptTemplate {
        system: None,
        user: template.to_string(),
    }
}

fn split_system_user_template(template: &str) -> Option<(String, String)> {
    const SYSTEM_TAG: &str = "---SYSTEM---";
    const USER_TAG: &str = "---USER---";

    let system_idx = template.find(SYSTEM_TAG)?;
    let user_idx = template.find(USER_TAG)?;
    if user_idx <= system_idx {
        return None;
    }

    let system_start = system_idx + SYSTEM_TAG.len();
    let user_start = user_idx + USER_TAG.len();
    let system = template[system_start..user_idx].trim().to_string();
    let user = template[user_start..].trim().to_string();
    Some((system, user))
}

fn extract_rust_code(raw: &str) -> Option<String> {
    let think_re = Regex::new(r"(?s)<think>.*?</think>").ok()?;
    let reasoning_re = Regex::new(r"(?s)<reasoning>.*?</reasoning>").ok()?;
    let fence_re = Regex::new(r"(?s)```(?:rust|rs|Rust|RS)?\s*\n(.*?)```").ok()?;

    let without_think = think_re.replace_all(raw, "");
    let cleaned = reasoning_re.replace_all(&without_think, "");

    let blocks: Vec<String> = fence_re
        .captures_iter(&cleaned)
        .filter_map(|caps| caps.get(1).map(|m| m.as_str().trim().to_string()))
        .filter(|block| !block.is_empty())
        .collect();

    let candidate = if blocks.is_empty() {
        strip_fallback_prose(cleaned.as_ref())
    } else if blocks.len() == 1 {
        blocks[0].clone()
    } else {
        blocks.join("\n\n")
    };

    let candidate = candidate.trim();
    if candidate.is_empty() {
        None
    } else {
        Some(candidate.to_string())
    }
}

fn strip_fallback_prose(raw: &str) -> String {
    let sentence_re = Regex::new(r#"^[A-Za-z][A-Za-z0-9 ,.!?'"`:-]*$"#).ok();
    let lines: Vec<&str> = raw.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    let mut start = 0usize;
    let mut end = lines.len();

    while start < end && is_prose_or_wrapper_line(lines[start], sentence_re.as_ref()) {
        start += 1;
    }
    while end > start && is_prose_or_wrapper_line(lines[end - 1], sentence_re.as_ref()) {
        end -= 1;
    }

    lines[start..end].join("\n")
}

fn is_prose_or_wrapper_line(line: &str, sentence_re: Option<&Regex>) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return true;
    }

    if trimmed.starts_with('#')
        || trimmed.starts_with('>')
        || trimmed.starts_with('*')
        || trimmed.starts_with('-')
        || trimmed.starts_with("```")
    {
        return true;
    }

    let has_rust_syntax = ['{', '}', '(', ')', ';'].iter().any(|c| trimmed.contains(*c));
    if has_rust_syntax {
        return false;
    }

    sentence_re.is_some_and(|re| re.is_match(trimmed))
}

async fn openrouter_chat(
    client: &reqwest::Client,
    cli: &Cli,
    system_prompt: Option<&str>,
    user_prompt: &str,
    temperature_override: Option<f32>,
    max_tokens_override: Option<u32>,
) -> Result<(String, Option<String>)> {
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
    let mut messages = Vec::new();
    if let Some(system_prompt) = system_prompt {
        messages.push(json!({ "role": "system", "content": system_prompt }));
    }
    messages.push(json!({ "role": "user", "content": user_prompt }));

    let body = json!({
        "model": cli.model,
        "messages": messages,
        "temperature": temperature_override.unwrap_or(cli.temperature),
        "max_tokens": max_tokens_override.unwrap_or(cli.max_tokens)
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
        .map(|c| c.message.content.clone().unwrap_or_default())
        .ok_or_else(|| anyhow!("OpenRouter response had no choices[0].message.content"))?;
    let finish_reason = parsed
        .choices
        .first()
        .and_then(|c| c.finish_reason.clone());

    if content.is_empty() {
        let usage = parsed.usage.as_ref();
        eprintln!(
            "OpenRouter empty content; finish_reason={:?}; model={:?}; usage(prompt={:?}, completion={:?}, total={:?}); raw={}",
            finish_reason,
            parsed.model,
            usage.and_then(|u| u.prompt_tokens),
            usage.and_then(|u| u.completion_tokens),
            usage.and_then(|u| u.total_tokens),
            text
        );
    }

    Ok((content, finish_reason))
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