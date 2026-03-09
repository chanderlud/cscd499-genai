import json
import logging
import os
import uuid
import re
from pathlib import Path
from collections import defaultdict
from typing import Any, Dict, Optional, Sequence, Mapping
import httpx

from langchain_core.messages import HumanMessage, ToolMessage, trim_messages
from langchain_ollama import ChatOllama


LOGGER = logging.getLogger(__name__)


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


def build_repair_message(eval_result: Dict[str, Any], main_rs: str) -> str:
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
    elif clippy_diagnostics and clippy_diagnostics.get("items") or 0 > 0:
        parts.append(
            "\n[clippy diagnostics]\n" + format_diagnostic_summary(clippy_diagnostics) + "\n Resolve clippy lints.")
    else:
        parts.append("BUILD AND TESTS PASSED. The code is ready for final submission")
        passed = True

    repair = "\n".join(parts)

    if passed:
        return repair
    else:
        answer = code_help_helper(main_rs, repair, "Explain how to fix the repair messages for the code.")
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


def code_help_helper(main_rs: str, context: str, question: str):
    main_rs = normalize_rust_text(main_rs, field_name="main_rs")
    run_id = uuid.uuid4().hex[:8]

    prompt = (
        "You are an expert Rust coder specializing in Windows API programming using the `windows` crate.\n\n"
        "## Task\n"
        "Review the provided code, referring to the context as needed. Answer the question in a consist manner "
        "while including useful information for the user. Do not implement tests, large blocks of code, "
        "or suggest logging as a solution."
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