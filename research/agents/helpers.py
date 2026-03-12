

def extract_rust_from_messages(messages):
    """Return the most recent fenced Rust code block from AI messages."""
    import re

    rust_block = re.compile(r"```(?:rust)?\s*(.*?)```", re.IGNORECASE | re.DOTALL)
    for msg in reversed(messages or []):
        is_ai = getattr(msg, "type", None) == "ai" or msg.__class__.__name__ == "AIMessage"
        if not is_ai:
            continue
        text = extract_message_text(msg)
        if not text:
            continue
        matches = list(rust_block.finditer(text))
        if matches:
            code = matches[-1].group(1).strip()
            if code:
                return code
    return None
import json
import logging
import os
import uuid
import re
from pathlib import Path
from collections import defaultdict
from typing import Any, Dict, Optional, Sequence, Mapping
import httpx
import time

from langchain_core.messages import HumanMessage, ToolMessage, trim_messages
from langchain_ollama import ChatOllama
from langchain_core.tools import tool


LOGGER = logging.getLogger(__name__)

IGNORABLE_UNUSED_CODES = frozenset({
    "unused_imports",
    "unused_variables",
    "dead_code",
    "unused_mut",
    "unused_assignments",
})


class FinalAnswerException(Exception):
    def __init__(self, main_rs: str):
        super().__init__("final_answer")
        self.main_rs = main_rs


def configure_logging(level: Optional[str] = None) -> None:
    resolved_name = (level or os.getenv("LOG_LEVEL", "INFO")).upper()
    resolved_level = getattr(logging, resolved_name, logging.INFO)
    root_logger = logging.getLogger()
    if not root_logger.handlers:
        logging.basicConfig(
            level=resolved_level,
            format="%(asctime)s %(levelname)s [%(name)s] %(message)s",
        )
    else:
        root_logger.setLevel(resolved_level)


def env(name: str, default: str) -> str:
    return os.getenv(name, default).rstrip("/")


def preview_text(value: Any, limit: int = 240) -> str:
    if isinstance(value, str):
        text = value
    else:
        try:
            text = json.dumps(value, ensure_ascii=False, sort_keys=True)
        except TypeError:
            text = repr(value)

    text = re.sub(r"\s+", " ", text).strip()
    if len(text) <= limit:
        return text
    return text[: limit - 16] + "... [truncated]"


def summarize_tool_args(tool_args: Dict[str, Any]) -> Dict[str, Any]:
    summary: Dict[str, Any] = {}
    for key, value in tool_args.items():
        if isinstance(value, str):
            summary[key] = {
                "type": "str",
                "len": len(value),
                "preview": preview_text(value, limit=120),
            }
        else:
            summary[key] = value
    return summary


def summarize_tool_output(tool_name: str, tool_out: str) -> str:
    if not isinstance(tool_out, str):
        return f"type={type(tool_out).__name__}"

    prefix = f"{tool_name} "
    if tool_out.startswith(prefix):
        return preview_text(tool_out, limit=240)

    try:
        payload = json.loads(tool_out)
    except Exception:
        return preview_text(tool_out, limit=240)

    if not isinstance(payload, dict):
        return preview_text(payload, limit=240)

    summary: Dict[str, Any] = {}
    for key in ("ok", "project_id", "success_criterion"):
        if key in payload:
            summary[key] = payload[key]

    for step_name in ("build", "clippy", "tests"):
        step = payload.get(step_name)
        if isinstance(step, dict):
            summary[f"{step_name}_ok"] = step.get("ok")
            summary[f"{step_name}_exit_code"] = step.get("exit_code")
            diagnostics = step.get("diagnostics")
            if isinstance(diagnostics, dict):
                summary[f"{step_name}_errors"] = diagnostics.get("errors")
                summary[f"{step_name}_warnings"] = diagnostics.get("warnings")

    results = payload.get("results")
    if isinstance(results, list):
        summary["results"] = len(results)

    return preview_text(summary or payload, limit=240)


def truncate_feedback(feedback: str, max_chars: int) -> str:
    if not isinstance(feedback, str):
        return ""
    if max_chars <= 0:
        return "... [truncated]"
    if len(feedback) <= max_chars:
        return feedback
    return feedback[-max_chars:] + "... [truncated]"


def compress_old_tool_messages(messages: Sequence[Any], keep_last_n: int = 1) -> list[Any]:
    copied = list(messages)
    if keep_last_n < 0:
        keep_last_n = 0

    tool_indexes: dict[str, list[int]] = defaultdict(list)
    for index, message in enumerate(copied):
        if isinstance(message, ToolMessage):
            tool_name = getattr(message, "name", None) or "unknown_tool"
            tool_indexes[tool_name].append(index)

    for tool_name, indexes in tool_indexes.items():
        stale_indexes = indexes[:-keep_last_n] if keep_last_n > 0 else indexes
        for stale_index in stale_indexes:
            original = copied[stale_index]
            original_text = extract_message_text(original)
            summary = summarize_tool_output(tool_name, original_text)
            copied[stale_index] = ToolMessage(
                content=f"[compressed previous {tool_name} output] {summary}",
                tool_call_id=original.tool_call_id,
                name=getattr(original, "name", None),
            )

    return copied


def apply_context_window(messages: Sequence[Any], max_tokens: int) -> list[Any]:
    def _approx_message_tokens(message: Any) -> int:
        if isinstance(message, dict):
            content = message.get("content", "")
            text = content if isinstance(content, str) else json.dumps(content, ensure_ascii=False)
        else:
            text = extract_message_text(message)

        # Roughly 1 token ~= 4 chars plus a small per-message envelope.
        return max(1, len(text) // 4) + 8

    def _approx_token_counter(message_list: Sequence[Any]) -> int:
        if not message_list:
            return 0
        return sum(_approx_message_tokens(message) for message in message_list)

    trimmed = trim_messages(
        list(messages),
        strategy="last",
        include_system=True,
        token_counter=_approx_token_counter,
        max_tokens=max_tokens,
        start_on="human",
        allow_partial=False,
    )
    original_human = None
    for message in messages:
        if isinstance(message, dict):
            if message.get("role") in {"human", "user"}:
                original_human = message
                break
            continue
        if getattr(message, "type", None) == "human":
            original_human = message
            break

    if original_human is None:
        return list(trimmed)

    if not trimmed:
        return [original_human]

    first = trimmed[0]
    first_is_human = (
                             isinstance(first, dict) and first.get("role") in {"human", "user"}
                     ) or (getattr(first, "type", None) == "human")

    if first_is_human:
        return list(trimmed)

    return [original_human, *list(trimmed)]


def extract_windows_path_reference(value: str) -> Optional[tuple[str, str]]:
    stripped = value.strip()
    match = re.match(
        r"^(?P<comment>//\s*)?(?P<path>(?:\\\\\?\\|\\\?\\)?[A-Za-z]:\\[^\r\n]*?\.rs)(?::(?P<rest>.*))?$",
        stripped,
        flags=re.DOTALL,
    )
    if not match:
        return None

    path_text = match.group("path")
    if path_text.startswith("\\?\\"):
        path_text = "\\" + path_text

    rest = match.group("rest") or ""
    return path_text, rest.lstrip()


def normalize_rust_text(value: str, *, field_name: str) -> str:
    if not isinstance(value, str):
        raise TypeError(f"{field_name} must be a string, got {type(value).__name__}")

    extracted = extract_windows_path_reference(value)
    if extracted is None:
        return value

    path_text, rest = extracted
    if rest:
        LOGGER.warning(
            "stripped_path_prefixed_text field=%s path=%s chars=%s",
            field_name,
            path_text,
            len(rest),
        )
        return rest

    return resolve_file_backed_text(path_text, field_name=field_name)


def resolve_file_backed_text(value: str, *, field_name: str) -> str:
    if not isinstance(value, str):
        raise TypeError(f"{field_name} must be a string, got {type(value).__name__}")

    normalized = extract_windows_path_reference(value)
    if normalized is not None:
        path_text, rest = normalized
        if rest:
            LOGGER.warning(
                "stripped_path_prefixed_text field=%s path=%s chars=%s",
                field_name,
                path_text,
                len(rest),
            )
            return rest
        value = path_text

    stripped = value.strip()
    if not stripped or "\n" in stripped or "\r" in stripped:
        return value

    path_candidates = []
    for candidate in (stripped, stripped.strip('"').strip("'")):
        if candidate and candidate not in path_candidates:
            path_candidates.append(candidate)
        normalized = candidate.replace("/", "\\")
        if normalized and normalized not in path_candidates:
            path_candidates.append(normalized)
        if normalized.startswith("\\\\?\\"):
            without_prefix = normalized[4:]
            if without_prefix and without_prefix not in path_candidates:
                path_candidates.append(without_prefix)

    for candidate in path_candidates:
        try:
            path = Path(candidate)
        except OSError:
            continue

        if not path.is_file():
            continue

        resolved = path.read_text(encoding="utf-8")
        LOGGER.warning(
            "resolved_file_backed_text field=%s path=%s chars=%s",
            field_name,
            path,
            len(resolved),
        )
        return resolved

    return value


def ensure_empty_main(main_rs: str) -> str:
    """
    The evaluator requires an empty main() present in the snippet.
    If user code already has main(), we leave it.
    Otherwise append fn main() {}.
    """
    if re.search(r"\bfn\s+main\s*\(", main_rs):
        return main_rs
    return main_rs.rstrip() + "\n\nfn main() {}\n"


def summarize_step(report: Dict[str, Any]) -> str:
    diag = report.get("diagnostics", {}) or {}
    errs = diag.get("errors", 0)
    warns = diag.get("warnings", 0)

    stdout = report.get("stdout", "") or ""
    stderr = report.get("stderr", "") or ""

    def tail(s: str, n: int = 4000) -> str:
        s = s.strip()
        return s[-n:] if len(s) > n else s

    return (
        f"{report.get('name', 'step')} ok={report.get('ok')} "
        f"timed_out={report.get('timed_out')} exit_code={report.get('exit_code')} "
        f"errors={errs} warnings={warns}\n"
        f"--- stderr (tail) ---\n{tail(stderr)}\n"
        f"--- stdout (tail) ---\n{tail(stdout)}\n"
    )


def summarize_eval(resp: Dict[str, Any]) -> str:
    parts = [f"GLOBAL ok={resp.get('ok')} project_id={resp.get('project_id')}"]
    for k in ["build", "clippy", "tests"]:
        if k in resp and isinstance(resp[k], dict):
            parts.append(summarize_step(resp[k]))
    return "\n\n".join(parts)


def extract_message_text(message: Any) -> str:
    content = getattr(message, "content", "")
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        parts = []
        for part in content:
            if isinstance(part, dict):
                text = part.get("text")
                if isinstance(text, str):
                    parts.append(text)
        return "\n".join(parts)
    return ""


def is_only_unused_warnings(eval_result: Dict[str, Any]) -> bool:
    build = eval_result.get("build") if isinstance(eval_result.get("build"), dict) else {}
    if not build.get("ok"):
        return False

    clippy = eval_result.get("clippy") if isinstance(eval_result.get("clippy"), dict) else {}

    build_diag = build.get("diagnostics") or {}
    clippy_diag = clippy.get("diagnostics") or {}

    if build_diag.get("errors", 0) > 0 or clippy_diag.get("errors", 0) > 0:
        return False

    combined_codes = set()
    for diag in (build_diag, clippy_diag):
        by_code = diag.get("by_code") or {}
        if isinstance(by_code, Mapping):
            combined_codes.update(by_code.keys())

    print(f"combined codes: {combined_codes}")

    if not combined_codes:
        return True

    if not combined_codes.issubset(IGNORABLE_UNUSED_CODES):
        return False

    for diag in (build_diag, clippy_diag):
        items = diag.get("items") or []
        if isinstance(items, Sequence) and not isinstance(items, (str, bytes)):
            for item in items:
                if isinstance(item, Mapping) and item.get("level") == "error":
                    return False

    return True


def build_repair_message(
    eval_result: Dict[str, Any], main_rs: str, problem_text: str = ""
) -> str:
    build = eval_result.get("build") if isinstance(eval_result.get("build"), dict) else {}
    clippy = eval_result.get("clippy") if isinstance(eval_result.get("clippy"), dict) else {}
    tests = eval_result.get("tests") if isinstance(eval_result.get("tests"), dict) else {}

    build_diagnostics = build.get("diagnostics") or {}
    clippy_diagnostics = clippy.get("diagnostics") or {}
    test_summary = tests.get("tests") or {}

    parts = []
    passed = False

    if not build.get("ok") or False:
        parts.append("BUILD FAILED. Fix the following errors before calling evaluate_rust again:")

        if build_diagnostics:
            parts.append("\n[build diagnostics]\n" + format_diagnostic_summary(build_diagnostics))
    elif not tests.get("ok") or False and test_summary:
        parts.append("TESTS FAILED. The unit tests that failed are: " + ", ".join(test_summary.get(
            "failed_names")) + ". Edit the code to pass the tests. Refer to the original problem for the exact requirements.")
    elif is_only_unused_warnings(eval_result):
        parts.append("Code is good to go. Only unused import/variable warnings remain, which are acceptable.")
        passed = True
    elif clippy_diagnostics and clippy_diagnostics.get("items") or 0 > 0:
        parts.append(
            "\n[clippy diagnostics]\n" + format_diagnostic_summary(clippy_diagnostics) + "\n Resolve clippy lints.")
    else:
        parts.append("BUILD AND TESTS PASSED. The code is ready for final submission")
        passed = True

    repair = "\n".join(parts)

    if not passed and repair and ("unresolved import" in repair.lower() or "cannot find" in repair.lower()):
        repair += "\n\n**Import error detected:** Call `rust_win_search(\"<symbol>\")` for each unresolved symbol to find its correct `windows` crate path before retrying."

    windows_unused_import_detected = False
    for diagnostics in (build_diagnostics, clippy_diagnostics):
        items = diagnostics.get("items") or []
        if not isinstance(items, Sequence) or isinstance(items, (str, bytes)):
            continue
        for item in items:
            if not isinstance(item, Mapping):
                continue
            code = str(item.get("code") or "")
            if code not in {"unused_imports", "E0unused", "dead_code"}:
                continue
            message = str(item.get("message") or "")
            rendered = str(item.get("rendered") or "")
            if "windows::" in message or "windows::" in rendered:
                windows_unused_import_detected = True
                break
        if windows_unused_import_detected:
            break

    if windows_unused_import_detected:
        repair += "\n\nUnused Windows API imports detected. Remove all `use windows::...` lines that are not referenced in a test assertion. Do not call `rust_win_search` to fix unused imports - simply delete them."

    if passed:
        return repair
    else:
        answer = code_help_helper(
            main_rs,
            repair,
            "Explain how to fix the repair messages for the code.",
            problem_text=problem_text,
        )
        return repair + "\n\n Expert repair suggestion: " + answer


def format_diagnostic_summary(summary: Mapping[str, Any]) -> str:
    def _as_int(value: Any, default: int = 0) -> int:
        try:
            return int(value)
        except (TypeError, ValueError):
            return default

    def _as_bool(value: Any) -> bool:
        return bool(value)

    def _as_str(value: Any, default: str = "") -> str:
        if value is None:
            return default
        return str(value)

    def _indent_block(text: str, prefix: str = "    ") -> str:
        return "\n".join(prefix + line if line else prefix.rstrip() for line in text.splitlines())

    errors = _as_int(summary.get("errors"))
    warnings = _as_int(summary.get("warnings"))
    notes = _as_int(summary.get("notes"))
    helps = _as_int(summary.get("helps"))
    truncated = _as_bool(summary.get("truncated"))

    by_code = summary.get("by_code") or {}
    if not isinstance(by_code, Mapping):
        by_code = {}

    items = summary.get("items") or []
    if not isinstance(items, Sequence) or isinstance(items, (str, bytes)):
        items = []

    lines: list[str] = []

    # Header
    lines.append("Diagnostic Summary")
    lines.append("==================")
    lines.append(f"errors: {errors}")
    lines.append(f"warnings: {warnings}")
    lines.append(f"notes: {notes}")
    lines.append(f"helps: {helps}")
    lines.append(f"items: {len(items)}")
    lines.append(f"truncated: {'yes' if truncated else 'no'}")

    # Codes section
    lines.append("")
    lines.append("By code")
    lines.append("-------")
    if by_code:
        for code, count in sorted(by_code.items(), key=lambda kv: (str(kv[0]), kv[1])):
            lines.append(f"{code}: {_as_int(count)}")
    else:
        lines.append("(none)")

    # Items section
    lines.append("")
    lines.append("Items")
    lines.append("-----")
    if items:
        for i, item in enumerate(items, start=1):
            if not isinstance(item, Mapping):
                lines.append(f"{i}. <invalid item: {item!r}>")
                continue

            level = _as_str(item.get("level"), "unknown")
            code = item.get("code")
            message = _as_str(item.get("message"))
            rendered = item.get("rendered")

            code_suffix = f" [{code}]" if code else ""
            lines.append(f"{i}. {level}{code_suffix}: {message}")

            if rendered:
                lines.append("   rendered:")
                lines.append(_indent_block(_as_str(rendered), prefix="     "))
    else:
        lines.append("(none)")

    return "\n".join(lines)


def refactor_with_specialist(main_rs: str, problem_text: str, run_id: str) -> str:
    model = ChatOllama(
        model="hf.co/Fortytwo-Network/Strand-Rust-Coder-14B-v1-GGUF:Q8_0",
        base_url=os.getenv("OLLAMA_BASE_URL", "http://127.0.0.1:11434"),
        temperature=0,
        num_predict=int(os.getenv("OLLAMA_NUM_PREDICT", "8000")),
    )
    prompt = (
        "Refactor the provided Rust code for idiomatic style, stronger error handling, reduced unsafe scope, "
        "and Clippy compliance, without changing observable behavior or breaking tests.\n\n"
        "Problem context:\n"
        f"{problem_text}\n\n"
        "Current src/main.rs:\n"
        "```rust\n"
        f"{main_rs}\n"
        "```\n\n"
        "Respond with only the refactored src/main.rs content inside a single ```rust ... ``` fence and nothing else."
    )
    try:
        response = model.invoke([HumanMessage(content=prompt)])
    except Exception as exc:
        LOGGER.warning(
            "refactor_with_specialist failed run_id=%s error=%s: %s",
            run_id,
            type(exc).__name__,
            exc,
        )
        return main_rs

    response_text = extract_message_text(response)
    match = re.search(r"```rust\s*(.*?)```", response_text, re.DOTALL)
    extracted = match.group(1).strip() if match else response_text.strip()
    if not extracted:
        LOGGER.warning(
            "refactor_with_specialist extraction_failed run_id=%s response_len=%s",
            run_id,
            len(response_text),
        )
        return main_rs

    LOGGER.info(
        "refactor_with_specialist run_id=%s before_len=%s after_len=%s",
        run_id,
        len(main_rs),
        len(extracted),
    )
    return extracted


def code_help_helper(
    main_rs: str,
    context: str,
    question: str,
    problem_text: str = "",
    doc_results: str = "",
):
    main_rs = normalize_rust_text(main_rs, field_name="main_rs")
    run_id = uuid.uuid4().hex[:8]

    sections = []
    if problem_text:
        sections.append("## Original Problem\n" f"{problem_text}\n\n")
    if doc_results:
        sections.append("## Documentation Tool Output\n" f"{doc_results}\n\n")
    optional_prefix = "".join(sections)

    prompt = (
        "You are an expert Rust coder specializing in Windows API programming using the `windows` crate.\n\n"
        "## Task\n"
        "Review the provided code, referring to the context as needed. Answer the question in a consist manner "
        "while including useful information for the user. Do not implement tests, large blocks of code, "
        "or suggest logging as a solution.\n\n"
        f"{optional_prefix}"
        "## Context\n"
        f"{context}\n\n"
        "## Question\n"
        f"{question}\n\n"
        "## Code Under Review\n"
        "```rust\n"
        f"{main_rs}\n"
        "```\n"
    )

    answer = code_help_tool(prompt, run_id)
    if not answer:
        return "code_help unavailable (OPENROUTER_API_KEY not set or request failed). Proceed with caution or call final_answer."
    LOGGER.info("code_help run_id=%s review_len=%s", run_id, len(answer))
    return answer


def code_help_tool(prompt: str, run_id: str) -> str:
    """Call OpenRouter /chat/completions for expert code review. Returns review text or '' on error."""
    api_key = os.getenv("OPENROUTER_API_KEY") or ""
    if not api_key:
        LOGGER.warning("review_code_with_openrouter skipped run_id=%s OPENROUTER_API_KEY not set", run_id)
        return ""
    base_url = env("OPENROUTER_BASE_URL", "https://openrouter.ai/api/v1")
    model = env("OPENROUTER_REVIEW_MODEL", "arcee-ai/trinity-large-preview:free")

    try:
        with httpx.Client(timeout=220.0) as client:
            r = client.post(
                f"{base_url}/chat/completions",
                headers={
                    "Authorization": f"Bearer {api_key}",
                    "Content-Type": "application/json",
                },
                json={
                    "model": model,
                    "messages": [{"role": "user", "content": prompt}],
                    "temperature": 0.2,
                    "max_tokens": 2048,
                },
            )
            r.raise_for_status()
            data = r.json()
            choices = data.get("choices") or []
            if choices and isinstance(choices[0], dict):
                msg = choices[0].get("message") or {}
                content = msg.get("content")
                if isinstance(content, str):
                    return content
            LOGGER.warning(
                "review_code_with_openrouter unexpected response run_id=%s keys=%s",
                run_id,
                list(data.keys()) if isinstance(data, dict) else "n/a",
            )
            return ""
    except Exception as e:
        LOGGER.warning(
            "review_code_with_openrouter failed run_id=%s error=%s: %s",
            run_id,
            type(e).__name__,
            e,
        )
        return ""

def build_tools(unit_tests_private: str, fixed_dependencies: str, _eval_state: Optional[dict] = None, run_tests: bool = True):
    eval_state = _eval_state if _eval_state is not None else {}
    msdocs_base = env("MSDOCS_BASE_URL", "http://127.0.0.1:3000")
    rustdocs_base = env("RUSTDOCS_BASE_URL", "http://127.0.0.1:3001")
    eval_base = env("RUST_EVAL_BASE_URL", "http://127.0.0.1:3002")

    client = httpx.Client(timeout=60.0)
    try:
        warmup_resp = client.get(f"{eval_base}/warmup", timeout=600)
        warmup_resp.raise_for_status()
        LOGGER.info(
            "eval warmup ok response=%s",
            preview_text(warmup_resp.json(), limit=160),
        )
    except Exception as exc:
        LOGGER.warning("eval warmup failed error=%s", exc)

    @tool("ms_doc_search")
    def ms_doc_search(
            q: str,
            scope: str = "win32",
            enrich: bool = True,
            top: int = 3,
            locale: str = "en-us",
            skip: int = 0,
            max_enrich: int = 8,
    ) -> str:
        """
        Search Microsoft Win32 documentation for a single Windows API symbol or exact API topic.

        Use this tool only when you need official Microsoft Win32 / C / C++ documentation for
        one specific Windows API item, such as a function, struct, constant, message, interface,
        macro, or header-defined type.

        Valid input:
        - A single API item name or exact API topic.
        - Usually just the symbol itself, for example:
          "CreateDirectoryW"
          "SECURITY_ATTRIBUTES"
          "WM_COPYDATA"
          "CreateFile"
          "HANDLE"

        Do NOT use this tool for:
        - General implementation questions
        - Natural-language questions
        - Multi-step behavior questions
        - Rust crate paths or Rust-specific API usage
        - Multiple items combined into one query
        - Queries that mix symbols with prose

        Never pass queries like:
        - "How do I create a directory and set permissions?"
        - "How do I watch a directory for changes in Rust?"
        - "CreateDirectoryW SECURITY_ATTRIBUTES example"
        - "difference between CreateFile and NtCreateFile"

        Prefer this tool over Rust tools only when the target is Win32/Microsoft docs and the
        needed result is about the official Windows API surface, signatures, flags, or behavior.

        Input rule:
        - Pass one symbol or one exact API topic only.
        - Keep the query short.
        - If the user asked a broad question, first identify the likely Win32 symbol, then search that symbol.

        Returns:
        - Search results from Microsoft Win32 docs, limited to the requested top count.
        """
        started = time.perf_counter()
        LOGGER.debug(
            "ms_doc_search request q=%r scope=%s top=%s enrich=%s locale=%s skip=%s max_enrich=%s",
            preview_text(q, limit=120),
            scope,
            top,
            enrich,
            locale,
            skip,
            max_enrich,
        )
        try:
            r = client.get(
                f"{msdocs_base}/v1/search",
                params={
                    "q": q,
                    "scope": scope,
                    "enrich": str(enrich).lower(),
                    "top": top,
                    "locale": locale,
                    "skip": skip,
                    "max_enrich": max_enrich,
                },
            )
            r.raise_for_status()
            data = r.json()
            if isinstance(data, dict) and "results" in data and isinstance(data["results"], list):
                data["results"] = data["results"][:top]
            duration_ms = int((time.perf_counter() - started) * 1000)
            LOGGER.info(
                "ms_doc_search ok duration_ms=%s results=%s q=%r",
                duration_ms,
                len(data.get("results", [])) if isinstance(data, dict) else "n/a",
                preview_text(q, limit=80),
            )
            return json.dumps(data, ensure_ascii=False, indent=2)[:18000]
        except Exception as e:
            duration_ms = int((time.perf_counter() - started) * 1000)
            LOGGER.exception(
                "ms_doc_search failed duration_ms=%s q=%r",
                duration_ms,
                preview_text(q, limit=80),
            )
            return f"ms_doc_search error: {type(e).__name__}: {e}"

    @tool("rust_win_search")
    def rust_win_search(item_name: str, limit: int = 10) -> str:
        """
        Search Rust Windows API docs for a single Windows API item as exposed by the Rust windows crate.

        Use this tool only to look up one Rust Windows item by name, typically a single Win32 API
        symbol such as a function, struct, enum, constant, or interface. This tool is for item lookup,
        not question answering.

        Valid input:
        - One item name only, for example:
          "CreateDirectoryW"
          "SECURITY_ATTRIBUTES"
          "PCWSTR"
          "HANDLE"

        If a path is available, only the final item name should be passed.
        Example:
        - Good: "CreateDirectoryW"
        - Not preferred: "Windows.Win32.Storage.FileSystem.CreateDirectoryW"

        Do NOT use this tool for:
        - General Rust questions
        - Natural-language questions
        - Multi-part queries
        - Combining function names with struct names
        - Queries containing prose, punctuation-heavy requests, or implementation goals
        - Looking up several items at once

        Never pass queries like:
        - "How do I call CreateDirectoryW from Rust?"
        - "CreateDirectoryW SECURITY_ATTRIBUTES"
        - "CreateDirectoryW and RemoveDirectoryW"
        - "How do I convert a string to PCWSTR?"
        - "best way to create a temp directory on Windows in Rust"

        Input rule:
        - Pass exactly one item name.
        - No explanation, no sentence, no extra keywords.
        - If the user asked a broad Rust/Windows question, first infer the most relevant item, then search only that item.

        Returns:
        - Matching Rust windows crate items and paths. This is a symbol lookup tool, not a reasoning tool.
        """

        q = item_name
        if "::" in q:
            q = q.split("::")[-1]

        started = time.perf_counter()
        LOGGER.debug(
            "rust_win_search request q=%r limit=%s",
            preview_text(q, limit=120),
            limit,
        )
        try:
            r = client.get(
                f"{rustdocs_base}/search",
                params={"q": q, "limit": limit},
            )
            r.raise_for_status()
            data = r.json()
            duration_ms = int((time.perf_counter() - started) * 1000)
            LOGGER.info(
                "rust_win_search ok duration_ms=%s results=%s q=%r",
                duration_ms,
                len(data) if isinstance(data, list) else len(data.get("results", [])) if isinstance(data,
                                                                                                    dict) else "n/a",
                preview_text(q, limit=80),
            )
            return json.dumps(data, ensure_ascii=False, indent=2)[:18000]
        except Exception as e:
            duration_ms = int((time.perf_counter() - started) * 1000)
            LOGGER.exception(
                "rust_win_search failed duration_ms=%s q=%r",
                duration_ms,
                preview_text(q, limit=80),
            )
            return f"rust_win_search error: {type(e).__name__}: {e}"

    @tool("format_rust")
    def format_rust(snippet: str) -> str:
        """
        Format Rust snippet using the formatting API. Returns formatted code (or error text).
        """
        snippet = normalize_rust_text(snippet, field_name="snippet")
        started = time.perf_counter()
        LOGGER.debug("format_rust request snippet_len=%s", len(snippet))
        try:
            r = client.post(f"{eval_base}/format", json={"snippet": snippet})
            r.raise_for_status()
            data = r.json()
            if data.get("ok") and data.get("formatted"):
                formatted = normalize_rust_text(data["formatted"], field_name="formatted")
                duration_ms = int((time.perf_counter() - started) * 1000)
                LOGGER.info(
                    "format_rust ok duration_ms=%s formatted_len=%s",
                    duration_ms,
                    len(formatted),
                )
                return formatted
            duration_ms = int((time.perf_counter() - started) * 1000)
            LOGGER.warning(
                "format_rust failed duration_ms=%s response=%s",
                duration_ms,
                preview_text(data, limit=200),
            )
            return f"format_rust failed: {json.dumps(data, ensure_ascii=False)[:12000]}"
        except Exception as e:
            duration_ms = int((time.perf_counter() - started) * 1000)
            LOGGER.exception("format_rust error duration_ms=%s snippet_len=%s", duration_ms, len(snippet))
            return f"format_rust error: {type(e).__name__}: {e}"

    @tool("evaluate_rust")
    def evaluate_rust(main_rs: str) -> str:
        """
        Build + clippy + run hidden tests. The harness appends hidden tests privately.
        Returns a human-readable failure summary plus the EvaluateResponse JSON.
        """
        main_rs = normalize_rust_text(main_rs, field_name="main_rs")
        print(main_rs)
        started = time.perf_counter()
        LOGGER.info(
            "evaluate_rust request main_rs_len=%s hidden_tests_len=%s",
            len(main_rs),
            len(unit_tests_private),
        )
        try:
            full_main = (
                    ensure_empty_main(main_rs).rstrip()
                    + "\n\n"
                    + unit_tests_private.strip()
                    + "\n"
            )
            r = client.post(
                f"{eval_base}/evaluate",
                json={"main_rs": full_main, "dependencies": fixed_dependencies, "run_tests": run_tests},
            )
            r.raise_for_status()
            data = r.json()
            duration_ms = int((time.perf_counter() - started) * 1000)
            LOGGER.info(
                "evaluate_rust ok duration_ms=%s summary=%s",
                duration_ms,
                summarize_tool_output("evaluate_rust", json.dumps(data, ensure_ascii=False)),
            )
            eval_state["last"] = data
            repair_message = build_repair_message(data, full_main)
            print(repair_message)
            return repair_message
        except Exception as e:
            duration_ms = int((time.perf_counter() - started) * 1000)
            LOGGER.exception("evaluate_rust error duration_ms=%s main_rs_len=%s", duration_ms, len(main_rs))
            return f"evaluate_rust error: {type(e).__name__}: {e}"

    @tool("code_review")
    def code_review(main_rs: str, problem_text: str) -> str:
        """
        Request an expert code review from an external model before calling final_answer.
        Use this after evaluate_rust returns ok=true to catch logic errors, unsafe issues,
        and Clippy violations that the build harness may not surface.
        Pass the full src/main.rs content and the original problem statement.
        Returns structured review feedback with a VERDICT, ISSUES, and SUGGESTED_FIXES.
        """
        main_rs = normalize_rust_text(main_rs, field_name="main_rs")
        run_id = uuid.uuid4().hex[:8]

        prompt = (
            "You are an expert Rust code reviewer specializing in Windows API programming using the `windows` crate.\n\n"
            "## Task\n"
            "Review the Rust code below for correctness, safety, and idiomatic style. The code is a solution to the problem "
            "described in the \"Problem\" section. Hidden unit tests will be appended and run against this code.\n\n"
            "## Review Criteria (address each in order)\n"
            "1. **Correctness** — Does the logic correctly solve the stated problem? Are Win32 API calls used with the right "
            "arguments, flags, and error-checking patterns?\n"
            "2. **Safety** — Is `unsafe` minimized and justified? Are raw pointers, handles, and lifetimes managed correctly? "
            "Are resources (HANDLEs, allocations) properly released?\n"
            "3. **Error handling** — Are all fallible Win32 calls checked? Is `windows::core::Result` / `HRESULT` propagated correctly?\n"
            "4. **Idiomatic Rust** — Does the code follow Rust conventions (ownership, borrowing, naming)? Are there unnecessary "
            "clones, unwraps, or panics?\n"
            "5. **Clippy compliance** — Would `cargo clippy` flag anything? List specific lints if so.\n"
            "6. **Import completeness** — Are all `use` paths present and correct for the `windows` crate features used?\n\n"
            "## Output Format\n"
            "Respond with a structured review using these exact headings:\n"
            "### VERDICT\n"
            "One of: APPROVE | NEEDS_CHANGES | REJECT\n"
            "(APPROVE = ready to submit; NEEDS_CHANGES = fixable issues found; REJECT = fundamental logic error)\n\n"
            "### ISSUES\n"
            "A numbered list of concrete issues. For each issue state: severity (CRITICAL/MAJOR/MINOR), location (line or construct), "
            "and a specific fix recommendation. If none, write \"None.\"\n\n"
            "### SUGGESTED_FIXES\n"
            "For each CRITICAL or MAJOR issue, describe the exact change needed in plain English. Do NOT rewrite the entire file.\n\n"
            "### SUMMARY\n"
            "One paragraph summarizing overall quality and the most important action to take next.\n\n"
            "## Problem\n"
            f"{problem_text}\n\n"
            "## Code Under Review\n"
            "```rust\n"
            f"{main_rs}\n"
            "```\n"
        )

        review = code_help_tool(prompt, run_id)
        if not review:
            return "code_review unavailable (OPENROUTER_API_KEY not set or request failed). Proceed with caution or call final_answer."
        LOGGER.info("code_review run_id=%s review_len=%s", run_id, len(review))
        return review

    @tool("code_help")
    def code_help(
        main_rs: str,
        context: str,
        question: str,
        problem_text: str = "",
        doc_results: str = "",
    ) -> str:
        """
        Request help from an outside expert. Provide the current code
        and a question. For example, ask how to implement a pattern,
        how to fix an error, how to use a certain API, etc. The context
        can include build errors or other information the outside expert
        will need to help answer the question.

        Always pass problem_text: the original problem statement (copy it
        verbatim from the user message). Always pass doc_results: paste
        the raw JSON/text output from any ms_doc_search or rust_win_search
        calls already made for the current problem, concatenated together.
        """
        answer = code_help_helper(
            main_rs, context, question, problem_text=problem_text, doc_results=doc_results
        )
        return answer

    @tool("final_answer")
    def final_answer(main_rs: str) -> str:
        """
        Submit the final src/main.rs content when it is formatted and ready.
        """
        main_rs = normalize_rust_text(main_rs, field_name="main_rs")
        if not isinstance(main_rs, str) or not main_rs.strip():
            raise ValueError("main_rs must be a non-empty string")
        raise FinalAnswerException(main_rs)

    # TODO re-enable microsoft doc search
    return [rust_win_search, format_rust, evaluate_rust, code_review, code_help, final_answer], eval_state
