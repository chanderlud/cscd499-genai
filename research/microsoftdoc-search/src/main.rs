use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use futures::{stream, StreamExt};
use moka::future::Cache;
use percent_encoding::percent_decode_str;
use regex::Regex;
use reqwest::Client;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use thiserror::Error;
use tokio::sync::Semaphore;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};
use url::Url;

#[derive(Clone)]
struct AppState {
    http: Client,
    // Cache extracted doc results by URL
    doc_cache: Cache<String, Arc<DocExtract>>,
    // Limit upstream requests (search + doc fetch)
    upstream_sem: Arc<Semaphore>,
    // Limit concurrent enrichment per request
    enrich_sem: Arc<Semaphore>,
}

#[derive(Debug, Serialize)]
struct SearchResponse {
    query: String,
    locale: String,
    scope: Scope,
    top: u32,
    skip: u32,
    fetched_at: DateTime<Utc>,
    total_count: u32,
    next_link: Option<String>,
    source: String,
    hits: Vec<SearchHit>,
}

#[derive(Debug, Serialize)]
struct SearchHit {
    title: String,
    url: String,
    description: String,
    last_updated: Option<DateTime<Utc>>,
    extracted: Option<DocExtract>,
}

/* -------------------------------- Errors -------------------------------- */

#[derive(Debug, Error)]
enum ApiError {
    #[error("bad request: {0}")]
    BadRequest(String),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Url(#[from] url::ParseError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, msg) = match &self {
            ApiError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
            ApiError::Reqwest(e) => (StatusCode::BAD_GATEWAY, e.to_string()),
            ApiError::Url(e) => (StatusCode::BAD_REQUEST, e.to_string()),
        };

        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/json".parse().unwrap());

        let body = serde_json::json!({
            "error": msg,
            "type": format!("{}", self),
        });

        (status, headers, Json(body)).into_response()
    }
}

#[derive(Debug, Deserialize)]
struct SearchParams {
    q: String,
    #[serde(default = "default_locale")]
    locale: String,
    #[serde(default = "default_top")]
    top: u32,
    #[serde(default)]
    skip: u32,
    #[serde(default = "default_scope")]
    scope: Scope,
    #[serde(default = "default_true")]
    enrich: bool,
    #[serde(default = "default_max_enrich")]
    max_enrich: usize,
}

#[derive(Debug, Deserialize)]
struct LearnSearchResponse {
    results: Vec<LearnSearchResult>,
    #[serde(default)]
    count: u32,
    #[serde(rename = "nextLink")]
    next_link: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LearnSearchResult {
    title: String,
    url: String,
    description: Option<String>,
    #[serde(rename = "lastUpdatedDate")]
    last_updated_date: Option<DateTime<Utc>>,
}

#[derive(Debug)]
struct SigParam {
    name: String,
    param_type: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "winapi_docsearch=info,tower_http=info".into()),
        )
        .init();

    let http = Client::builder()
        .user_agent("winapi-docsearch/0.1 (+https://learn.microsoft.com)")
        .timeout(Duration::from_secs(20))
        .build()
        .expect("reqwest client");

    let ttl_secs: u64 = std::env::var("CACHE_TTL_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(60 * 60);

    let state = AppState {
        http,
        doc_cache: Cache::builder()
            .time_to_live(Duration::from_secs(ttl_secs))
            .max_capacity(10_000)
            .build(),
        upstream_sem: Arc::new(Semaphore::new(16)),
        enrich_sem: Arc::new(Semaphore::new(6)),
    };

    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/v1/search", get(search_handler))
        .route("/v1/doc", get(doc_handler))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let bind = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".into());
    info!("listening on http://{}", bind);

    let listener = tokio::net::TcpListener::bind(&bind).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum Scope {
    Win32,
    Drivers,
    All,
}

async fn search_handler(
    State(state): State<AppState>,
    Query(p): Query<SearchParams>,
) -> Result<Json<SearchResponse>, ApiError> {
    let raw = learn_search(&state, &p.q, &p.locale, p.top, p.skip).await?;

    // Filter to useful Windows API pages unless user asked for everything
    let hits: Vec<LearnSearchResult> = raw
        .results
        .into_iter()
        .filter(|r| match p.scope {
            Scope::Win32 => r.url.contains("/windows/win32/"),
            Scope::Drivers => r.url.contains("/windows-hardware/drivers/"),
            Scope::All => true,
        })
        .collect();

    let mut out_hits: Vec<SearchHit> = hits
        .into_iter()
        .map(|r| SearchHit {
            title: r.title,
            url: r.url,
            description: r.description.unwrap_or_default(),
            last_updated: r.last_updated_date,
            extracted: None,
        })
        .collect();

    if p.enrich {
        let max_n = p.max_enrich.min(out_hits.len());
        let targets = out_hits[..max_n]
            .iter()
            .map(|h| h.url.clone())
            .collect::<Vec<_>>();

        let extracted = stream::iter(targets)
            .map(|url| {
                let st = state.clone();
                async move {
                    let _permit = st.enrich_sem.acquire().await.expect("semaphore");
                    extract_doc_cached(&st, &url).await.map(|d| (url, d))
                }
            })
            .buffer_unordered(8)
            .collect::<Vec<_>>()
            .await;

        for res in extracted {
            match res {
                Ok((url, doc)) => {
                    if let Some(hit) = out_hits.iter_mut().find(|h| h.url == url) {
                        hit.extracted = Some((*doc).clone());
                    }
                }
                Err(e) => warn!("enrich failed: {}", e),
            }
        }
    }

    Ok(Json(SearchResponse {
        query: p.q,
        locale: p.locale,
        scope: p.scope,
        top: p.top,
        skip: p.skip,
        fetched_at: Utc::now(),
        hits: out_hits,
        // pass through next link if present
        next_link: raw.next_link,
        total_count: raw.count,
        source: "https://learn.microsoft.com/api/search".to_string(),
    }))
}

#[derive(Debug, Deserialize)]
struct DocParams {
    url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DocExtract {
    url: String,
    title: String,
    summary: Option<String>,

    // From Syntax section
    signature: Option<String>,
    return_type: Option<String>,
    function_name: Option<String>,

    parameters: Vec<ParamDoc>,
    return_value: Option<String>,
    remarks: Option<String>,
    requirements: Option<Vec<RequirementKV>>,
    examples: Vec<ExampleBlock>,
    see_also: Vec<LinkRef>,

    // Pre-chewed blocks for LLM context
    context_blocks: Vec<ContextBlock>,

    extracted_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ParamDoc {
    name: String,
    param_type: Option<String>,
    direction: Option<Vec<String>>, // in/out/optional
    description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct RequirementKV {
    key: String,
    value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ExampleBlock {
    language: Option<String>,
    description: Option<String>,
    code: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct LinkRef {
    text: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ContextBlock {
    kind: String,
    priority: u8, // 1..10
    content: String,
}

async fn doc_handler(
    State(state): State<AppState>,
    Query(p): Query<DocParams>,
) -> Result<Json<DocExtract>, ApiError> {
    let url = percent_decode_str(&p.url).decode_utf8_lossy().to_string();
    let doc = extract_doc_cached(&state, &url).await?;
    Ok(Json((*doc).clone()))
}

async fn learn_search(
    state: &AppState,
    query: &str,
    locale: &str,
    top: u32,
    skip: u32,
) -> Result<LearnSearchResponse, ApiError> {
    // Endpoint is widely used but not really "officially documented" as an API,
    // so keep this conservative.
    let base = "https://learn.microsoft.com/api/search";
    let filter = "(category eq 'Documentation')";

    let _permit = state.upstream_sem.acquire().await.expect("semaphore");

    let resp = state
        .http
        .get(base)
        .query(&[
            ("search", query),
            ("locale", locale),
            ("scoringprofile", "semantic-answers"),
            ("$top", &top.to_string()),
            ("$skip", &skip.to_string()),
            ("$filter", filter),
            ("expandScope", "true"),
            ("partnerId", "LearnSite"),
        ])
        .send()
        .await?
        .error_for_status()?;

    Ok(resp.json::<LearnSearchResponse>().await?)
}

/* -------------------------- Doc extraction/cache ------------------------- */

async fn extract_doc_cached(state: &AppState, url: &str) -> Result<Arc<DocExtract>, ApiError> {
    if let Some(v) = state.doc_cache.get(url).await {
        return Ok(v);
    }
    let doc = Arc::new(extract_doc(state, url).await?);
    state.doc_cache.insert(url.to_string(), doc.clone()).await;
    Ok(doc)
}

async fn extract_doc(state: &AppState, url: &str) -> Result<DocExtract, ApiError> {
    let url = normalize_url(url)?;
    enforce_allowlist(&url)?;

    let _permit = state.upstream_sem.acquire().await.expect("semaphore");

    let html = state
        .http
        .get(url.as_str())
        .header("accept-language", "en-US,en;q=0.9")
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    parse_learn_doc(url.as_str(), &html)
}

fn parse_learn_doc(url: &str, html: &str) -> Result<DocExtract, ApiError> {
    let doc = Html::parse_document(html);

    let h1_sel = Selector::parse("main h1, article h1, h1").unwrap();
    let title = doc
        .select(&h1_sel)
        .next()
        .map(|h| norm_text(h.text().collect::<String>()))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Untitled".to_string());

    // Most Learn content sits under <main>…<article>…,
    // but fall back to body if the DOM is weird.
    let article_sel = Selector::parse("main article, article, main, body").unwrap();
    let article = doc.select(&article_sel).next();

    let summary = article.as_ref().and_then(|a| first_paragraph_after_h1(a));

    // Extract section text
    let syntax = article
        .as_ref()
        .and_then(|a| extract_section_text(a, "Syntax"));
    let params_section = article
        .as_ref()
        .and_then(|a| extract_section_node(a, "Parameters"));
    let return_value = article
        .as_ref()
        .and_then(|a| extract_section_text(a, "Return value"));
    let remarks = article
        .as_ref()
        .and_then(|a| extract_section_text(a, "Remarks"));
    let requirements = article.as_ref().and_then(|a| extract_requirements(a));
    let examples = article
        .as_ref()
        .map(|a| extract_examples(a))
        .unwrap_or_default();
    let see_also = article
        .as_ref()
        .map(|a| extract_see_also(a))
        .unwrap_or_default();

    // Parse signature into return type + fn name + param types (best-effort)
    let (return_type, function_name, sig_params) = syntax
        .as_deref()
        .map(parse_signature)
        .unwrap_or((None, None, vec![]));

    let mut param_type_map = std::collections::HashMap::new();
    for sp in sig_params {
        param_type_map.insert(sp.name.clone(), sp.param_type);
    }

    let parameters = if let Some(ps) = params_section {
        parse_parameters_from_section(ps, &param_type_map)
    } else {
        vec![]
    };

    let mut context_blocks = Vec::new();
    if let Some(sig) = &syntax {
        context_blocks.push(ContextBlock {
            kind: "signature".to_string(),
            priority: 10,
            content: sig.clone(),
        });
    }
    if !parameters.is_empty() {
        let mut s = String::new();
        for p in &parameters {
            s.push_str(&format!(
                "- {}{}{}: {}\n",
                p.name,
                p.param_type
                    .as_ref()
                    .map(|t| format!(" ({})", t))
                    .unwrap_or_default(),
                p.direction
                    .as_ref()
                    .map(|d| format!(" [{}]", d.join(", ")))
                    .unwrap_or_default(),
                one_line(&p.description),
            ));
        }
        context_blocks.push(ContextBlock {
            kind: "parameters".to_string(),
            priority: 10,
            content: s,
        });
    }
    if let Some(rv) = &return_value {
        context_blocks.push(ContextBlock {
            kind: "return_value".to_string(),
            priority: 9,
            content: rv.clone(),
        });
    }
    if let Some(rm) = &remarks {
        context_blocks.push(ContextBlock {
            kind: "remarks".to_string(),
            priority: 6,
            content: rm.clone(),
        });
    }
    if !examples.is_empty() {
        // Keep examples tight: first 2 blocks for context
        let mut s = String::new();
        for ex in examples.iter().take(2) {
            s.push_str(&format!(
                "```{}\n{}\n```\n",
                ex.language.clone().unwrap_or_default(),
                ex.code.trim()
            ));
        }
        context_blocks.push(ContextBlock {
            kind: "examples".to_string(),
            priority: 8,
            content: s,
        });
    }

    Ok(DocExtract {
        url: url.to_string(),
        title,
        summary,

        signature: syntax,
        return_type,
        function_name,

        parameters,
        return_value,
        remarks,
        requirements,
        examples,
        see_also,

        context_blocks,
        extracted_at: Utc::now(),
    })
}

fn first_paragraph_after_h1(article: &ElementRef<'_>) -> Option<String> {
    // Find first non-empty <p> inside the main content area.
    let p_sel = Selector::parse("p").unwrap();
    article
        .select(&p_sel)
        .map(|p| norm_text(p.text().collect::<String>()))
        .find(|t| t.len() > 20) // avoid tiny junk lines
}

fn extract_section_node<'a>(article: &'a ElementRef<'a>, heading: &str) -> Option<ElementRef<'a>> {
    let h2_sel = Selector::parse("h2").unwrap();
    article.select(&h2_sel).find(|h| {
        let t = norm_text(h.text().collect::<String>());
        t.eq_ignore_ascii_case(heading)
    })
}

fn extract_section_text(article: &ElementRef<'_>, heading: &str) -> Option<String> {
    let h2 = extract_section_node(article, heading)?;
    let mut out = String::new();

    // Walk siblings until next H2
    let mut node = h2.next_sibling();
    while let Some(n) = node {
        if let Some(el) = ElementRef::wrap(n) {
            let name = el.value().name();
            if name.eq_ignore_ascii_case("h2") {
                break;
            }
            if name.eq_ignore_ascii_case("pre") {
                // keep code blocks
                let code = el.text().collect::<String>();
                if !code.trim().is_empty() {
                    out.push_str(code.trim());
                    out.push('\n');
                }
            } else if name.eq_ignore_ascii_case("p")
                || name.eq_ignore_ascii_case("ul")
                || name.eq_ignore_ascii_case("ol")
            {
                let t = norm_text(el.text().collect::<String>());
                if !t.is_empty() {
                    out.push_str(&t);
                    out.push('\n');
                }
            }
        }
        node = n.next_sibling();
    }

    let out = out.trim().to_string();
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn parse_signature(sig: &str) -> (Option<String>, Option<String>, Vec<SigParam>) {
    // Very rough parsing, but good enough for Win32 typedef-y signatures.
    // Example lines:
    // BOOL WriteFile(
    //   [in] HANDLE hFile,
    //   ...
    // );
    let lines: Vec<String> = sig
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    if lines.is_empty() {
        return (None, None, vec![]);
    }

    // First line: "<RET> <NAME>("
    let first = &lines[0];
    let first = first.trim_end_matches('(').trim();
    let mut parts = first.split_whitespace().collect::<Vec<_>>();
    if parts.len() < 2 {
        return (None, None, vec![]);
    }
    let func_name = parts.pop().map(|s| s.to_string());
    let ret_type = Some(parts.join(" "));

    let bracket_re = Regex::new(r"^\s*\[[^\]]+\]\s*").unwrap();

    let mut params = Vec::new();
    for l in lines.iter().skip(1) {
        if l.starts_with(");") || l == ");" || l.ends_with(");") {
            break;
        }
        let mut l = l.trim_end_matches(',').trim().to_string();
        l = bracket_re.replace(&l, "").to_string();
        // Now something like: "HANDLE hFile" or "LPCVOID lpBuffer"
        let toks = l.split_whitespace().collect::<Vec<_>>();
        if toks.len() >= 2 {
            let name = toks[toks.len() - 1].to_string();
            let ty = toks[..toks.len() - 1].join(" ");
            params.push(SigParam {
                name,
                param_type: Some(ty),
            });
        }
    }

    (ret_type, func_name, params)
}

fn parse_parameters_from_section(
    h2: ElementRef<'_>,
    sig_types: &std::collections::HashMap<String, Option<String>>,
) -> Vec<ParamDoc> {
    // Win32 docs often format parameter entries as:
    // <p><code>[in] hFile</code></p>
    // <p>Description...</p>
    // <p>More...</p>
    let code_sel = Selector::parse("code").unwrap();
    let header_re =
        Regex::new(r"^\[(?P<attrs>[^\]]+)\]\s*(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*$").unwrap();

    let mut out: Vec<ParamDoc> = vec![];
    let mut node = h2.next_sibling();

    let mut current: Option<ParamDoc> = None;

    while let Some(n) = node {
        if let Some(el) = ElementRef::wrap(n) {
            let name = el.value().name();
            if name.eq_ignore_ascii_case("h2") {
                break;
            }
            if name.eq_ignore_ascii_case("p") {
                // Is this a param header line?
                if let Some(code) = el.select(&code_sel).next() {
                    let code_text = norm_text(code.text().collect::<String>());
                    if let Some(cap) = header_re.captures(&code_text) {
                        // flush previous
                        if let Some(c) = current.take() {
                            out.push(c);
                        }
                        let attrs = cap["attrs"]
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect::<Vec<_>>();
                        let pname = cap["name"].to_string();
                        let ptype = sig_types.get(&pname).cloned().flatten();

                        current = Some(ParamDoc {
                            name: pname,
                            param_type: ptype,
                            direction: if attrs.is_empty() { None } else { Some(attrs) },
                            description: String::new(),
                        });
                        node = n.next_sibling();
                        continue;
                    }
                }

                // Otherwise, treat as description text
                let t = norm_text(el.text().collect::<String>());
                if let Some(cur) = current.as_mut() {
                    if !t.is_empty() {
                        if !cur.description.is_empty() {
                            cur.description.push('\n');
                        }
                        cur.description.push_str(&t);
                    }
                }
            } else if name.eq_ignore_ascii_case("ul") || name.eq_ignore_ascii_case("ol") {
                // bullet details belong to current param
                let t = norm_text(el.text().collect::<String>());
                if let Some(cur) = current.as_mut() {
                    if !t.is_empty() {
                        if !cur.description.is_empty() {
                            cur.description.push('\n');
                        }
                        cur.description.push_str(&t);
                    }
                }
            } else if name.eq_ignore_ascii_case("pre") {
                // ignore code blocks inside Parameters, usually not needed
            } else if name.eq_ignore_ascii_case("table") {
                // sometimes params are tables; not handled here
            }
        }
        node = n.next_sibling();
    }

    if let Some(c) = current.take() {
        out.push(c);
    }

    // Clean empties
    out.into_iter()
        .filter(|p| !p.name.is_empty())
        .map(|mut p| {
            p.description = p.description.trim().to_string();
            p
        })
        .collect()
}

fn extract_examples(article: &ElementRef<'_>) -> Vec<ExampleBlock> {
    // Find H2 "Examples" then gather <pre><code>
    let h2_sel = Selector::parse("h2").unwrap();
    let pre_sel = Selector::parse("pre").unwrap();
    let code_sel = Selector::parse("code").unwrap();

    let h2 = article.select(&h2_sel).find(|h| {
        let t = norm_text(h.text().collect::<String>());
        t.eq_ignore_ascii_case("Examples") || t.eq_ignore_ascii_case("Example")
    });

    let Some(h2) = h2 else {
        return vec![];
    };

    let mut out = vec![];
    let mut node = h2.next_sibling();

    while let Some(n) = node {
        if let Some(el) = ElementRef::wrap(n) {
            if el.value().name().eq_ignore_ascii_case("h2") {
                break;
            }
            if el.value().name().eq_ignore_ascii_case("pre") {
                let code = el
                    .select(&code_sel)
                    .next()
                    .map(|c| c.text().collect::<String>())
                    .unwrap_or_else(|| el.text().collect::<String>());

                let lang = el
                    .select(&code_sel)
                    .next()
                    .and_then(|c| c.value().attr("class").map(|s| s.to_string()))
                    .and_then(|cls| detect_lang(&cls));

                if !code.trim().is_empty() {
                    out.push(ExampleBlock {
                        language: lang,
                        description: None,
                        code: code.trim().to_string(),
                    });
                }
            } else {
                // In case examples are nested further down
                for pre in el.select(&pre_sel) {
                    let code = pre
                        .select(&code_sel)
                        .next()
                        .map(|c| c.text().collect::<String>())
                        .unwrap_or_else(|| pre.text().collect::<String>());
                    if !code.trim().is_empty() {
                        out.push(ExampleBlock {
                            language: None,
                            description: None,
                            code: code.trim().to_string(),
                        });
                    }
                }
            }
        }
        node = n.next_sibling();
    }

    out
}

fn detect_lang(class_attr: &str) -> Option<String> {
    // common patterns: "lang-cpp", "language-cpp", etc
    for part in class_attr.split_whitespace() {
        if let Some(rest) = part.strip_prefix("lang-") {
            return Some(rest.to_string());
        }
        if let Some(rest) = part.strip_prefix("language-") {
            return Some(rest.to_string());
        }
    }
    None
}

fn extract_see_also(article: &ElementRef<'_>) -> Vec<LinkRef> {
    let h2_sel = Selector::parse("h2").unwrap();
    let a_sel = Selector::parse("a").unwrap();

    let h2 = article.select(&h2_sel).find(|h| {
        let t = norm_text(h.text().collect::<String>());
        t.eq_ignore_ascii_case("See also")
    });

    let Some(h2) = h2 else {
        return vec![];
    };

    let mut out = vec![];
    let mut node = h2.next_sibling();
    while let Some(n) = node {
        if let Some(el) = ElementRef::wrap(n) {
            if el.value().name().eq_ignore_ascii_case("h2") {
                break;
            }
            for a in el.select(&a_sel) {
                if let Some(href) = a.value().attr("href") {
                    let text = norm_text(a.text().collect::<String>());
                    if !text.is_empty() {
                        out.push(LinkRef {
                            text,
                            url: absolutize_learn_href(href),
                        });
                    }
                }
            }
        }
        node = n.next_sibling();
    }
    out
}

fn extract_requirements(article: &ElementRef<'_>) -> Option<Vec<RequirementKV>> {
    // Many Learn pages put Requirements in a table.
    let h2_sel = Selector::parse("h2").unwrap();
    let table_sel = Selector::parse("table").unwrap();
    let tr_sel = Selector::parse("tr").unwrap();
    let td_sel = Selector::parse("td").unwrap();

    let h2 = article.select(&h2_sel).find(|h| {
        let t = norm_text(h.text().collect::<String>());
        t.eq_ignore_ascii_case("Requirements")
    })?;

    let mut node = h2.next_sibling();
    while let Some(n) = node {
        if let Some(el) = ElementRef::wrap(n) {
            if el.value().name().eq_ignore_ascii_case("h2") {
                break;
            }
            if let Some(table) = el.select(&table_sel).next() {
                let mut rows = vec![];
                for tr in table.select(&tr_sel) {
                    let cells = tr.select(&td_sel).collect::<Vec<_>>();
                    if cells.len() >= 2 {
                        let k = norm_text(cells[0].text().collect::<String>());
                        let v = norm_text(cells[1].text().collect::<String>());
                        if !k.is_empty() && !v.is_empty() {
                            rows.push(RequirementKV { key: k, value: v });
                        }
                    }
                }
                if !rows.is_empty() {
                    return Some(rows);
                }
            }
        }
        node = n.next_sibling();
    }
    None
}

fn normalize_url(u: &str) -> Result<Url, ApiError> {
    // Accept either full URL or a Learn-relative path
    if u.starts_with("http://") || u.starts_with("https://") {
        Ok(Url::parse(u)?)
    } else if u.starts_with('/') {
        Ok(Url::parse(&format!("https://learn.microsoft.com{}", u))?)
    } else {
        Err(ApiError::BadRequest(
            "url must be absolute or learn.microsoft.com-relative".into(),
        ))
    }
}

fn enforce_allowlist(u: &Url) -> Result<(), ApiError> {
    let host = u.host_str().unwrap_or("");
    if host != "learn.microsoft.com" {
        return Err(ApiError::BadRequest(
            "only learn.microsoft.com URLs are allowed".into(),
        ));
    }
    Ok(())
}

fn absolutize_learn_href(href: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        href.to_string()
    } else if href.starts_with('/') {
        format!("https://learn.microsoft.com{}", href)
    } else {
        // relative path
        format!(
            "https://learn.microsoft.com/{}",
            href.trim_start_matches("./")
        )
    }
}

fn norm_text(s: String) -> String {
    // collapse whitespace
    let mut out = String::with_capacity(s.len());
    let mut last_space = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !last_space {
                out.push(' ');
                last_space = true;
            }
        } else {
            out.push(ch);
            last_space = false;
        }
    }
    out.trim().to_string()
}

fn one_line(s: &str) -> String {
    norm_text(s.to_string())
}

fn default_locale() -> String {
    "en-us".to_string()
}
fn default_top() -> u32 {
    5
}
fn default_max_enrich() -> usize {
    3
}
fn default_true() -> bool {
    true
}
fn default_scope() -> Scope {
    Scope::Win32
}
