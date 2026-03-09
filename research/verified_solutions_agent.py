import json
import logging
import os
import re
import time
import uuid
from dataclasses import dataclass
from pathlib import Path
from collections import defaultdict
from typing import Any, Dict, Optional, Mapping, Sequence

import httpx

from langchain.agents import create_agent
from langchain_core.messages import HumanMessage, ToolMessage, trim_messages
from langchain_core.tools import tool
from langchain_ollama import ChatOllama

FIXED_DEPENDENCIES = """
regex = "*"
rand = "*"
md5 = "*"
windows = { version = "0.62.2", features = [
    "Win32_System_Com",
    "Win32_UI",
    "Win32_UI_Shell",
    "Win32_System_Ole",
    "Win32_System_WindowsProgramming",
    "Win32_System_SystemInformation",
    "Win32_Storage",
    "Win32_Storage_FileSystem",
    "Win32_Security",
    "Win32_System_Pipes",
    "Win32_System_Threading",
    "Win32_System_IO"] }
""".strip() + "\n"

LOGGER = logging.getLogger(__name__)


class FinalAnswerException(Exception):
    def __init__(self, main_rs: str):
        super().__init__("final_answer")
        self.main_rs = main_rs


def _env(name: str, default: str) -> str:
    return os.getenv(name, default).rstrip("/")


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


def _preview_text(value: Any, limit: int = 240) -> str:
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


def _summarize_tool_args(tool_args: Dict[str, Any]) -> Dict[str, Any]:
    summary: Dict[str, Any] = {}
    for key, value in tool_args.items():
        if isinstance(value, str):
            summary[key] = {
                "type": "str",
                "len": len(value),
                "preview": _preview_text(value, limit=120),
            }
        else:
            summary[key] = value
    return summary


def _summarize_tool_output(tool_name: str, tool_out: str) -> str:
    if not isinstance(tool_out, str):
        return f"type={type(tool_out).__name__}"

    prefix = f"{tool_name} "
    if tool_out.startswith(prefix):
        return _preview_text(tool_out, limit=240)

    try:
        payload = json.loads(tool_out)
    except Exception:
        return _preview_text(tool_out, limit=240)

    if not isinstance(payload, dict):
        return _preview_text(payload, limit=240)

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

    return _preview_text(summary or payload, limit=240)


def _truncate_feedback(feedback: str, max_chars: int) -> str:
    if not isinstance(feedback, str):
        return ""
    if max_chars <= 0:
        return "... [truncated]"
    if len(feedback) <= max_chars:
        return feedback
    return feedback[-max_chars:] + "... [truncated]"


def _compress_old_tool_messages(messages: Sequence[Any], keep_last_n: int = 1) -> list[Any]:
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
            original_text = _extract_message_text(original)
            summary = _summarize_tool_output(tool_name, original_text)
            copied[stale_index] = ToolMessage(
                content=f"[compressed previous {tool_name} output] {summary}",
                tool_call_id=original.tool_call_id,
                name=getattr(original, "name", None),
            )

    return copied


def _apply_context_window(messages: Sequence[Any], model: Any, max_tokens: int) -> list[Any]:
    def _approx_message_tokens(message: Any) -> int:
        if isinstance(message, dict):
            content = message.get("content", "")
            text = content if isinstance(content, str) else json.dumps(content, ensure_ascii=False)
        else:
            text = _extract_message_text(message)

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


def _extract_windows_path_reference(value: str) -> Optional[tuple[str, str]]:
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


def _normalize_rust_text(value: str, *, field_name: str) -> str:
    if not isinstance(value, str):
        raise TypeError(f"{field_name} must be a string, got {type(value).__name__}")

    extracted = _extract_windows_path_reference(value)
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

    return _resolve_file_backed_text(path_text, field_name=field_name)


def _resolve_file_backed_text(value: str, *, field_name: str) -> str:
    if not isinstance(value, str):
        raise TypeError(f"{field_name} must be a string, got {type(value).__name__}")

    normalized = _extract_windows_path_reference(value)
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


def _ensure_empty_main(main_rs: str) -> str:
    """
    The evaluator requires an empty main() present in the snippet.
    If user code already has main(), we leave it.
    Otherwise append fn main() {}.
    """
    if re.search(r"\bfn\s+main\s*\(", main_rs):
        return main_rs
    return main_rs.rstrip() + "\n\nfn main() {}\n"


def _summarize_step(report: Dict[str, Any]) -> str:
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


def _summarize_eval(resp: Dict[str, Any]) -> str:
    parts = [f"GLOBAL ok={resp.get('ok')} project_id={resp.get('project_id')}"]
    for k in ["build", "clippy", "tests"]:
        if k in resp and isinstance(resp[k], dict):
            parts.append(_summarize_step(resp[k]))
    return "\n\n".join(parts)


def _build_repair_message(eval_result: Dict[str, Any]) -> str:
    build = eval_result.get("build") if isinstance(eval_result.get("build"), dict) else {}
    clippy = eval_result.get("clippy") if isinstance(eval_result.get("clippy"), dict) else {}
    tests = eval_result.get("tests") if isinstance(eval_result.get("tests"), dict) else {}

    build_diagnostics = build.get("diagnostics") or {}
    clippy_diagnostics = clippy.get("diagnostics") or {}
    test_summary = tests.get("tests") or {}

    parts = []

    if not build.get("ok") or False:
        parts.append("BUILD FAILED. Fix the following errors before calling evaluate_rust again:")
    elif not tests.get("ok") or False:
        parts.append("TESTS FAILED. Fix the following errors before calling evaluate_rust again:")

    if build_diagnostics:
        parts.append("\n[build diagnostics]\n" + format_diagnostic_summary(build_diagnostics))

    if clippy_diagnostics:
        parts.append("\n[clippy diagnostics]\n" + format_diagnostic_summary(clippy_diagnostics))

    parts.append(
        "\nYou MUST add missing use statements, fix unresolved imports or undeclared types, and resubmit corrected code. Do not forget the Windows Crate Hints."
    )

    if test_summary:
        parts.append("\n[test summary]\n" + json.dumps(test_summary))

        if test_summary.get("failed") or 0 > 0:
            parts.append("You MUST modify the code to pass the unit tests in `failed_names` use the names as hints of what behavior is incorrect in the current code.")

    if not build_diagnostics and not clippy_diagnostics:
        parts.append(
            "\nNo build/clippy diagnostics were available. Inspect the evaluation JSON and correct the code before retrying."
        )

    return "\n".join(parts)


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


def _extract_json_object(text: str) -> Dict[str, Any]:
    if not isinstance(text, str):
        raise TypeError(f"Expected string output, got {type(text).__name__}")

    text = text.strip()
    if not text:
        raise ValueError("Empty output")

    try:
        return json.loads(text)
    except json.JSONDecodeError:
        pass

    sanitized = _sanitize_json_text(text)
    try:
        return json.loads(sanitized)
    except json.JSONDecodeError:
        pass

    fence_match = re.search(r"```(?:json)?\s*(\{.*?\})\s*```", text, re.DOTALL)
    if fence_match:
        fenced = fence_match.group(1)
        try:
            return json.loads(fenced)
        except json.JSONDecodeError:
            return json.loads(_sanitize_json_text(fenced))

    decoder = json.JSONDecoder()
    for candidate in (text, sanitized):
        for idx, char in enumerate(candidate):
            if char != "{":
                continue
            try:
                obj, _ = decoder.raw_decode(candidate[idx:])
            except json.JSONDecodeError:
                continue
            if isinstance(obj, dict):
                return obj

    raise ValueError("Could not extract a JSON object from model output")


def _sanitize_json_text(text: str) -> str:
    out = []
    in_string = False
    i = 0
    while i < len(text):
        ch = text[i]

        if not in_string:
            out.append(ch)
            if ch == '"':
                in_string = True
            i += 1
            continue

        if ch == "\\":
            if i + 1 >= len(text):
                out.append("\\\\")
                i += 1
                continue

            nxt = text[i + 1]
            if nxt in {'"', "\\", "/", "b", "f", "n", "r", "t", "u"}:
                out.append(ch)
                out.append(nxt)
            else:
                out.append("\\\\")
                out.append(nxt)
            i += 2
            continue

        if ch == "\n":
            out.append("\\n")
            i += 1
            continue

        if ch == "\r":
            out.append("\\r")
            i += 1
            continue

        if ch == "\t":
            out.append("\\t")
            i += 1
            continue

        out.append(ch)
        if ch == '"':
            in_string = False
        i += 1

    return "".join(out)


def _extract_message_text(message: Any) -> str:
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


def _is_tool_request(obj: Dict[str, Any]) -> bool:
    return (
        isinstance(obj, dict)
        and isinstance(obj.get("name"), str)
        and isinstance(obj.get("arguments"), dict)
    )


def _is_final_answer(obj: Dict[str, Any]) -> bool:
    return isinstance(obj, dict) and "main_rs" in obj and isinstance(obj["main_rs"], str)


def build_tools(unit_tests_private: str):
    msdocs_base = _env("MSDOCS_BASE_URL", "http://127.0.0.1:3000")
    rustdocs_base = _env("RUSTDOCS_BASE_URL", "http://127.0.0.1:3001")
    eval_base = _env("RUST_EVAL_BASE_URL", "http://127.0.0.1:3002")

    client = httpx.Client(timeout=30.0)

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
            _preview_text(q, limit=120),
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
                _preview_text(q, limit=80),
            )
            return json.dumps(data, ensure_ascii=False, indent=2)[:18000]
        except Exception as e:
            duration_ms = int((time.perf_counter() - started) * 1000)
            LOGGER.exception(
                "ms_doc_search failed duration_ms=%s q=%r",
                duration_ms,
                _preview_text(q, limit=80),
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
            _preview_text(q, limit=120),
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
                len(data) if isinstance(data, list) else len(data.get("results", [])) if isinstance(data, dict) else "n/a",
                _preview_text(q, limit=80),
            )
            return json.dumps(data, ensure_ascii=False, indent=2)[:18000]
        except Exception as e:
            duration_ms = int((time.perf_counter() - started) * 1000)
            LOGGER.exception(
                "rust_win_search failed duration_ms=%s q=%r",
                duration_ms,
                _preview_text(q, limit=80),
            )
            return f"rust_win_search error: {type(e).__name__}: {e}"

    @tool("format_rust")
    def format_rust(snippet: str) -> str:
        """
        Format Rust snippet using the formatting API. Returns formatted code (or error text).
        """
        snippet = _normalize_rust_text(snippet, field_name="snippet")
        started = time.perf_counter()
        LOGGER.debug("format_rust request snippet_len=%s", len(snippet))
        try:
            r = client.post(f"{eval_base}/format", json={"snippet": snippet})
            r.raise_for_status()
            data = r.json()
            if data.get("ok") and data.get("formatted"):
                formatted = _normalize_rust_text(data["formatted"], field_name="formatted")
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
                _preview_text(data, limit=200),
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
        main_rs = _normalize_rust_text(main_rs, field_name="main_rs")
        print(main_rs)
        started = time.perf_counter()
        LOGGER.info(
            "evaluate_rust request main_rs_len=%s hidden_tests_len=%s",
            len(main_rs),
            len(unit_tests_private),
        )
        try:
            full_main = (
                _ensure_empty_main(main_rs).rstrip()
                + "\n\n"
                + unit_tests_private.strip()
                + "\n"
            )
            r = client.post(
                f"{eval_base}/evaluate",
                json={"main_rs": full_main, "dependencies": FIXED_DEPENDENCIES},
            )
            r.raise_for_status()
            data = r.json()
            duration_ms = int((time.perf_counter() - started) * 1000)
            LOGGER.info(
                "evaluate_rust ok duration_ms=%s summary=%s",
                duration_ms,
                _summarize_tool_output("evaluate_rust", json.dumps(data, ensure_ascii=False)),
            )
            repair_message = _build_repair_message(data)
            print(repair_message)
            return repair_message
        except Exception as e:
            duration_ms = int((time.perf_counter() - started) * 1000)
            LOGGER.exception("evaluate_rust error duration_ms=%s main_rs_len=%s", duration_ms, len(main_rs))
            return f"evaluate_rust error: {type(e).__name__}: {e}"

    @tool("final_answer")
    def final_answer(main_rs: str) -> str:
        """
        Submit the final src/main.rs content when it is formatted and ready.
        """
        main_rs = _normalize_rust_text(main_rs, field_name="main_rs")
        if not isinstance(main_rs, str) or not main_rs.strip():
            raise ValueError("main_rs must be a non-empty string")
        raise FinalAnswerException(main_rs)

    return [ms_doc_search, rust_win_search, format_rust, evaluate_rust, final_answer]


def build_agent(tools):
    model_name = os.getenv("OLLAMA_MODEL", "glm-4.7-flash:latest")
    model_url = os.getenv("OLLAMA_BASE_URL", "http://127.0.0.1:11434")

    model = ChatOllama(
        model=model_name,
        base_url=model_url,
        temperature=0,
        num_predict=int(os.getenv("OLLAMA_NUM_PREDICT", "8000")),
    )
    context_limit = int(os.getenv("OLLAMA_CONTEXT_TOKENS", "6000"))

    system_prompt = """You solve Win32/Windows API programming problems in Rust.

Hard rules:
- Use ms_doc_search and rust_win_search to confirm any Win32 API signature/behavior and the correct Rust windows crate path/features.
- Write stable Rust that compiles as src/main.rs. Do NOT include tests. The judge will append hidden tests after your code.
- You may call evaluate_rust to run build/clippy/tests. Keep iterating until it reports ok=true.
- When evaluate_rust returns, read the build diagnostics carefully. If errors say unresolved import or use of undeclared type, add the missing use statements at the top of the file and call evaluate_rust again with corrected code.
- Do NOT call evaluate_rust again with identical code after a failed build. You must make a concrete repair first.
- Only before the final output (once evaluate_rust passes without build errors), call format_rust on the final src/main.rs and use the formatted result.
- When you have a complete, formatted, tested solution, call the final_answer tool with the full src/main.rs content.

Quality rules:
- Prefer safe wrappers. If unsafe is required, minimize scope and justify via comments.
- Always include this import at top of the output (even if unused):
  #[allow(unused_imports)]
  use windows::core::{Result, Error};
- Utilize error propagation on API methods that return Result.
- When API methods do not return Result directly, use the appropriate pattern to check the return for success, get the last error if needed, and return an error.
- Dependencies are fixed and MUST NOT be output by the model.
- The windows, rand, md5, and regex crates are available for use. The rust_win_search tool provides import paths in the windows crate.

Windows Crate Hints:
- Prefer using the W variants of functions
- Functions that accept a `windows_core::Param<T>`` expect `Some(&T)`` as the parameter
- Functions that accept a specific Option<T> may have Some(T) or None as the parameter

The following helper may be used to create wide strings
```
use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(iter::once(0)).collect()
}

fn example() {
     let wide_str = wide_null(OsStr::new(r""));
     SomeFunctionW(Some(&PCWSTR(wide_str.as_ptr()));
}
```
"""

    LOGGER.info(
        "build_agent model=%s base_url=%s tool_count=%s",
        model_name,
        model_url,
        len(tools),
    )

    agent = create_agent(
        model=model,
        tools=tools,
        system_prompt=system_prompt,
    )

    return agent.with_config({"recursion_limit": 40}), model, context_limit


@dataclass
class SolveResult:
    main_rs: str
    last_eval: Dict[str, Any]


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

    response_text = _extract_message_text(response)
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


def solve_problem(problem_text: str, unit_tests_private: str, max_attempts: int = 6) -> SolveResult:
    run_id = uuid.uuid4().hex[:8]
    tools = build_tools(unit_tests_private)
    agent, agent_model, context_limit = build_agent(tools)
    tool_map = {t.name: t for t in tools}

    feedback = ""
    last_eval: Dict[str, Any] = {}
    refactor_done = False

    LOGGER.info(
        "solve_problem start run_id=%s problem_len=%s hidden_tests_len=%s max_attempts=%s",
        run_id,
        len(problem_text),
        len(unit_tests_private),
        max_attempts,
    )

    for attempt in range(1, max_attempts + 1):
        LOGGER.info(
            "solve_problem attempt_start run_id=%s attempt=%s feedback_len=%s",
            run_id,
            attempt,
            len(feedback),
        )
        run_input = problem_text if not feedback else (
            problem_text + "\n\n---\nREPAIR FEEDBACK:\n" + _truncate_feedback(feedback, 3000)
        )

        messages = [{"role": "user", "content": run_input}]
        main_rs: Optional[str] = None
        last_submitted_main_rs: Optional[str] = None

        for tool_hops in range(40):
            invoke_started = time.perf_counter()
            messages = _compress_old_tool_messages(messages, keep_last_n=1)
            messages = _apply_context_window(messages, model=agent_model, max_tokens=context_limit)
            try:
                result = agent.invoke({"messages": messages})
            except FinalAnswerException as answer:
                LOGGER.info(
                    "solve_problem final_answer_exception run_id=%s attempt=%s hop=%s main_rs_len=%s",
                    run_id,
                    attempt,
                    tool_hops,
                    len(answer.main_rs),
                )
                main_rs = answer.main_rs
                break
            duration_ms = int((time.perf_counter() - invoke_started) * 1000)

            msgs = result.get("messages") or []
            print(msgs)
            last = msgs[-1] if msgs else None
            content = _extract_message_text(last)
            LOGGER.debug(
                "agent_invoke ok run_id=%s attempt=%s hop=%s duration_ms=%s message_count=%s response_preview=%r",
                run_id,
                attempt,
                tool_hops,
                duration_ms,
                len(msgs),
                _preview_text(content, limit=240),
            )
            try:
                obj = _extract_json_object(content)
            except Exception:
                LOGGER.exception(
                    "agent_output_parse_failed run_id=%s attempt=%s hop=%s response_preview=%r",
                    run_id,
                    attempt,
                    tool_hops,
                    _preview_text(content, limit=400),
                )
                raise

            if _is_tool_request(obj):
                tool_name = obj["name"]
                tool_args = obj["arguments"]

                if tool_name not in tool_map:
                    raise RuntimeError(f"Model requested unknown tool: {tool_name}")

                if tool_name == "evaluate_rust":
                    submitted_main_rs = tool_args.get("main_rs")
                    if (
                        isinstance(submitted_main_rs, str)
                        and last_submitted_main_rs is not None
                        and submitted_main_rs == last_submitted_main_rs
                    ):
                        tool_out = (
                            "ERROR: You submitted the same code again. The previous evaluation already failed. "
                            "You MUST modify the code (e.g., add missing imports) before re-evaluating."
                        )
                        LOGGER.warning(
                            "tool_request_duplicate_submission run_id=%s attempt=%s hop=%s tool=%s",
                            run_id,
                            attempt,
                            tool_hops,
                            tool_name,
                        )
                        messages = list(msgs) + [
                            ToolMessage(
                                content=tool_out,
                                tool_call_id=f"manual_{tool_hops}",
                                name=tool_name,
                            )
                        ]
                        continue

                LOGGER.info(
                    "tool_request run_id=%s attempt=%s hop=%s tool=%s args=%s",
                    run_id,
                    attempt,
                    tool_hops,
                    tool_name,
                    _preview_text(_summarize_tool_args(tool_args), limit=280),
                )
                tool_started = time.perf_counter()
                try:
                    if tool_name == "evaluate_rust" and isinstance(tool_args.get("main_rs"), str):
                        last_submitted_main_rs = tool_args["main_rs"]
                    tool_out = tool_map[tool_name].invoke(tool_args)
                except FinalAnswerException as answer:
                    LOGGER.info(
                        "tool_final_answer_exception run_id=%s attempt=%s hop=%s tool=%s main_rs_len=%s",
                        run_id,
                        attempt,
                        tool_hops,
                        tool_name,
                        len(answer.main_rs),
                    )
                    main_rs = answer.main_rs
                    break
                tool_duration_ms = int((time.perf_counter() - tool_started) * 1000)

                LOGGER.info(
                    "tool_result run_id=%s attempt=%s hop=%s tool=%s duration_ms=%s summary=%s",
                    run_id,
                    attempt,
                    tool_hops,
                    tool_name,
                    tool_duration_ms,
                    _summarize_tool_output(tool_name, tool_out),
                )

                messages = list(msgs) + [
                    ToolMessage(
                        content=tool_out,
                        tool_call_id=f"manual_{tool_hops}",
                        name=tool_name,
                    )
                ]
                continue

            if _is_final_answer(obj):
                LOGGER.info(
                    "solve_problem final_answer_json run_id=%s attempt=%s hop=%s main_rs_len=%s",
                    run_id,
                    attempt,
                    tool_hops,
                    len(obj["main_rs"]),
                )
                main_rs = obj["main_rs"]
                break

            raise RuntimeError("Agent finished without calling final_answer.")
        else:
            raise RuntimeError("Too many tool hops without producing a final answer.")

        if main_rs is None:
            raise RuntimeError("Agent finished without calling final_answer.")

        LOGGER.info(
            "solve_problem format_candidate run_id=%s attempt=%s main_rs_len=%s",
            run_id,
            attempt,
            len(main_rs),
        )
        formatted = tool_map["format_rust"].invoke({"snippet": main_rs})
        if isinstance(formatted, str) and not (
            formatted.startswith("format_rust error") or formatted.startswith("format_rust failed")
        ):
            LOGGER.info(
                "solve_problem format_applied run_id=%s attempt=%s before_len=%s after_len=%s",
                run_id,
                attempt,
                len(main_rs),
                len(formatted),
            )
            main_rs = formatted
        else:
            LOGGER.warning(
                "solve_problem format_skipped run_id=%s attempt=%s summary=%s",
                run_id,
                attempt,
                _summarize_tool_output("format_rust", formatted),
            )

        eval_json = tool_map["evaluate_rust"].invoke({"main_rs": main_rs})
        try:
            last_eval = _extract_json_object(eval_json)
        except Exception:
            LOGGER.exception(
                "solve_problem eval_json_parse_failed run_id=%s attempt=%s raw=%r",
                run_id,
                attempt,
                _preview_text(eval_json, limit=400),
            )
            last_eval = {"ok": False, "raw": eval_json}

        if last_eval.get("ok") is True:
            LOGGER.info(
                "solve_problem success run_id=%s attempt=%s summary=%s",
                run_id,
                attempt,
                _preview_text(last_eval, limit=240),
            )
            if not refactor_done:
                main_rs = refactor_with_specialist(main_rs, problem_text, run_id)
                refactor_done = True
                feedback = "Refactored for code quality. Re-verify correctness."
                continue
            return SolveResult(
                main_rs=main_rs.rstrip() + "\n",
                last_eval=last_eval,
            )

        feedback = _build_repair_message(last_eval)
        LOGGER.warning(
            "solve_problem attempt_failed run_id=%s attempt=%s feedback=%r",
            run_id,
            attempt,
            _preview_text(feedback, limit=400),
        )

    LOGGER.error(
        "solve_problem exhausted_attempts run_id=%s max_attempts=%s last_eval=%r",
        run_id,
        max_attempts,
        _preview_text(last_eval, limit=400),
    )
    raise RuntimeError(
        f"Failed to solve within {max_attempts} attempts. "
        f"Last eval:\n{_summarize_eval(last_eval)}"
    )


if __name__ == "__main__":
    configure_logging()
    problem_text = open(
        "dataset/simple-winapi/problems/0a5d7328-0ec4-4088-83d2-7e1c0e8b27c7.md",
        "r",
        encoding="utf-8",
    ).read()
    unit_tests_text = open(
        "dataset/simple-winapi/tests/0a5d7328-0ec4-4088-83d2-7e1c0e8b27c7.rs",
        "r",
        encoding="utf-8",
    ).read()

    r = solve_problem(problem_text, unit_tests_text)
    print(r)