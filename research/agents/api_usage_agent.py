import argparse
import concurrent.futures
import json
import os
import re
import threading
import uuid
from collections import defaultdict
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

import httpx

from helpers import (
    LOGGER,
    StepRecorder,
    batch_rustdoc_lookup,
    build_repair_context,
    configure_logging,
    env,
    eval_server_evaluate,
    eval_server_format,
    eval_server_warmup,
    extract_rust_code_block,
    extract_symbols_from_diagnostics,
    extract_windows_api_symbols,
    openrouter_generate_code,
    preview_text,
)


def parse_enabled_features(deps_text: str) -> set[str]:
    match = re.search(
        r'windows\s*=\s*\{.*?features\s*=\s*\[(.*?)\].*?\}',
        deps_text,
        re.DOTALL,
    )
    if match is None:
        return set()
    return set(re.findall(r'"([^"]+)"', match.group(1)))


FIXED_DEPENDENCIES = (
    Path(__file__).resolve().parent.parent / "rust_dependencies.md"
).read_text(encoding="utf-8")
ENABLED_WINDOWS_FEATURES: set[str] = parse_enabled_features(FIXED_DEPENDENCIES)

MANIFEST_SYSTEM_PROMPT = """Manifest generation is code-driven for this agent.
No LLM call is used during the manifest phase.
"""

SOLUTION_SYSTEM_PROMPT = """You are an expert Rust engineer specializing in Win32/Windows API programming using the `windows` crate.

Output requirements:
- Output exactly one complete `src/main.rs` inside a single ```rust code fence.
- No explanation text outside the fence.
- Do not include tests.
- Do not include `fn main()`.
- Use only stable Rust.

Code constraints:
- Use the `windows` crate for Win32 APIs and prefer W-suffix APIs where applicable.
- Minimize `unsafe` scope to the smallest possible block.
- Use proper error propagation for HRESULT/WIN32_ERROR patterns.
- Keep behavior focused on the problem requirements.
- Include this import near the top:
  use windows::core::{Result, Error};

Available crates:
- windows
- rand
- md5
- regex
- tempfile
- sha2

Useful helper pattern:
```rust
fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}
```

Quality rules:
- Use `?` operator for Result-returning calls.
- After a failing non-`Result` Win32 call, call `windows::core::Error::from_thread()` — no argument — to capture `GetLastError()` as a `windows::core::Error`.
- To convert a raw `u32` error code to an `HRESULT`, call `HRESULT::from_win32(code)` — one `u32` argument. Never call `Error::from_win32(code)` with an argument; that method does not exist.
- Do not try to use the `?` operator with HRESULT, instead convert to `windows::core::Error` with windows::core::Error::from_hresult(hresult), then use `?`.
- Minimize `unsafe` blocks; justify each with a comment.
- The snippet must compile and pass clippy with no warnings (deny(warnings) is enforced).

## Win32 Error-Handling Reference

### `HRESULT` / `WIN32_ERROR` conversions (for known error codes)

When you have a raw Win32 error code (`u32`) or a `WIN32_ERROR` value and need an `HRESULT`,
use the following patterns (do NOT construct HRESULT literals manually):

```rust
use windows::core::HRESULT;
use windows::Win32::Foundation::{WIN32_ERROR, ERROR_ACCESS_DENIED};

// From a raw u32 code (e.g. returned by GetLastError as a u32):
fn from_raw_code(code: u32) -> HRESULT {
    HRESULT::from_win32(code)
}

// From a WIN32_ERROR value:
fn from_win32_error(err: WIN32_ERROR) -> HRESULT {
    err.to_hresult()
    // equivalent: HRESULT::from(err)
}

// Inline example — converting a known constant:
fn example() -> HRESULT {
    HRESULT::from_win32(ERROR_ACCESS_DENIED.0)
}
```

Key rules:
- `WIN32_ERROR` has a `.0` field (the raw `u32`); pass it to `HRESULT::from_win32()` when you need the raw value.
- Prefer `err.to_hresult()` over `.0` when you already hold a `WIN32_ERROR` — it is more idiomatic.
- Never hard-code `HRESULT(0x80070005_u32 as i32)` or similar literals.

### DO NOT CONFUSE these APIs

// WRONG — Error has no from_win32(code) method:
//   windows::core::Error::from_win32(code)
//
// CORRECT — use HRESULT::from_win32 for a code, Error::from_thread() for GetLastError:
//   HRESULT::from_win32(code)          // takes a u32 argument
//   windows::core::Error::from_hresult(hresult) // takes a HRESULT argument
//   windows::core::Error::from_thread() // zero arguments — reads GetLastError()

### `windows::core::Error` — reads `GetLastError()` automatically (zero arguments)

Use this after a failing non-`Result` Win32 call:
- `windows::core::Error::from_win32()` takes **no arguments**.
- It captures the current thread's `GetLastError()` value as a `windows::core::Error`.

## Win32 Threading Rules
- Do NOT add `unsafe impl Send` or `unsafe impl Sync` to any struct that wraps a `HANDLE`, raw pointer, or COM interface. This is unsound.
- Explicitly forbid `unsafe impl Send` / `unsafe impl Sync` on any wrapper that holds a `HANDLE` or raw pointer.
- Approved strategies (use in this priority order):
  1. **Create the HANDLE inside the thread closure**: The handle is only needed on the worker thread - open/create it there, use it, and close it before the closure returns.
  2. **Pass the raw integer value across the thread boundary**: Extract the handle as `isize` (via `.0`) before `thread::spawn`, move the integer into the closure, and reconstruct the typed handle inside the closure with `HANDLE(raw)`.
  3. **Use a channel to send work descriptions, not handles**: Send file paths, IDs, or other `Send` data to the thread; let the thread open its own handle.
  4. **Use `std::thread::scope`**: When the thread must not outlive the current stack frame, a scoped thread can borrow non-`Send` data safely.

## Task Style
- The problem is a simple API usage exercise: implement a single function that calls one Windows API method with given parameter values and returns `windows::core::Result<T>`.
- Do not invent complex logic. Focus on correct API call, parameter passing, and result handling.
"""

REPAIR_PROMPT_TEMPLATE = """Your previous code attempt failed to compile/pass tests. Fix ONLY the reported errors.
Use the Rustdoc symbol resolution below to correct any import paths.
Do NOT rewrite from scratch - make targeted fixes.

{context}

Windows repair reminders:
- To wrap a raw `u32` code as an `HRESULT`, use `HRESULT::from_win32(code)` (one argument) or `err.to_hresult()` for a `WIN32_ERROR` value.
- To capture `GetLastError()` as a `windows::core::Error`, use `windows::core::Error::from_thread()` with no argument. `Error::from_win32` does not accept a `u32` argument — that overload does not exist.
- Prefer W-suffix Win32 APIs and minimize `unsafe` scope.
- If diagnostics contain "cannot be sent between threads safely" or "the trait Send is not implemented", this is a thread-safety capture issue.
- Do NOT introduce `unsafe impl Send` / `unsafe impl Sync`. That is unsound and will not be accepted.
- Refactor using one of these strategies (in order):
  1. **Create the HANDLE inside the thread closure**: if only the worker needs it, open/create there, use it, close before return.
  2. **Pass a raw integer value across the boundary**: extract handle `.0` as `isize` before `thread::spawn`, move that integer, reconstruct typed `HANDLE(raw)` inside the closure.
  3. **Send work descriptions via channel instead of handles**: pass paths/IDs/other `Send` data, and open the handle on the worker.
  4. **Use `std::thread::scope`**: when the thread does not outlive the current frame, use scoped threads to borrow safely.

Output the complete fixed src/main.rs in a single ```rust code fence.
"""

PROBLEM_GENERATION_USER_TEMPLATE = """## Target API
- Name: {api_name}
- Path: {api_path}
- Signature: `{api_signature}`

## Related Types
{related_types_block}

## Variation
{variation}

## Task
Write a function named `{wrapper_name}` with this wrapper signature:

```rust
{wrapper_signature}
```

{variation_instructions}

Requirements:
- Call `{api_name}` directly.
- Use concrete parameter values rather than forwarding parameters from the wrapper.
- Keep the code short and focused on a single API call.
- Use the extracted signature and related type information above for imports and type handling.
- Output only a single complete `src/main.rs` in a Rust code fence.
"""


@dataclass
class ManifestEntry:
    id: str
    api_name: str
    api_path: str
    api_signature: str
    related_items: List[Dict[str, Any]]
    variation: str
    status: str
    problem_id: Optional[str] = None


@dataclass
class GeneratedProblem:
    entry_id: str
    problem_md: str
    main_rs: str
    last_eval: Dict[str, Any]
    verified: bool = True


@dataclass
class ScanSummary:
    files_scanned: int = 0
    pub_fn_declarations: int = 0
    link_wrappers: int = 0
    feature_enabled_wrappers: int = 0


def _error_score(eval_result: Dict[str, Any]) -> int:
    build = eval_result.get("build") if isinstance(eval_result.get("build"), dict) else {}
    clippy = eval_result.get("clippy") if isinstance(eval_result.get("clippy"), dict) else {}
    tests = eval_result.get("tests") if isinstance(eval_result.get("tests"), dict) else {}

    def diag_errors(stage: Dict[str, Any]) -> int:
        diagnostics = stage.get("diagnostics") if isinstance(stage.get("diagnostics"), dict) else {}
        try:
            return int(diagnostics.get("errors", 0))
        except (TypeError, ValueError):
            return 0

    score = diag_errors(build) + diag_errors(clippy)
    tests_info = tests.get("tests") if isinstance(tests.get("tests"), dict) else {}
    try:
        score += int(tests_info.get("failed", 0))
    except (TypeError, ValueError):
        pass
    if eval_result.get("ok") is True:
        score = 0
    return score


def _first_diagnostic_hint(eval_result: Dict[str, Any]) -> str:
    for stage in ("build", "clippy"):
        stage_data = eval_result.get(stage)
        if not isinstance(stage_data, dict):
            continue
        diagnostics = stage_data.get("diagnostics")
        if not isinstance(diagnostics, dict):
            continue
        items = diagnostics.get("items")
        if not isinstance(items, list):
            continue
        for item in items:
            if not isinstance(item, dict):
                continue
            message = item.get("message")
            if isinstance(message, str) and message.strip():
                return message.strip()
    return "the top compiler error"


def _is_feature_not_enabled_error(eval_result: Dict[str, Any]) -> bool:
    build = eval_result.get("build")
    if not isinstance(build, dict):
        return False
    diagnostics = build.get("diagnostics")
    if not isinstance(diagnostics, dict):
        return False
    items = diagnostics.get("items")
    if not isinstance(items, list):
        return False
    for item in items:
        if not isinstance(item, dict):
            continue
        message = item.get("message")
        if not isinstance(message, str):
            continue
        lowered = message.lower()
        if "found an item that was configured out" in item.get("rendered"):
            return True
        if "use of unstable library feature" in lowered:
            return True
        if "feature is not enabled" in lowered or "feature not enabled" in lowered:
            return True
        if "requires the" in lowered and "feature" in lowered:
            return True
    return False


def _camel_to_snake(name: str) -> str:
    step_one = re.sub(r"(.)([A-Z][a-z]+)", r"\1_\2", name)
    step_two = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", step_one)
    return step_two.lower()


def _extract_return_type(signature: str) -> str:
    if "->" not in signature:
        return "()"
    return signature.split("->", 1)[1].strip().rstrip(";")


def _normalize_base_wrapper_return_type(api_signature: str) -> str:
    raw = _extract_return_type(api_signature)
    compact = raw.replace(" ", "")
    if compact.startswith("windows::core::Result<"):
        return raw
    if compact.startswith("Result<"):
        return f"windows::core::{raw}"
    return f"windows::core::Result<{raw}>"


def _wrapper_name(entry: ManifestEntry) -> str:
    return f"call_{_camel_to_snake(entry.api_name)}"


def _wrapper_signature(entry: ManifestEntry) -> str:
    wrapper_name = _wrapper_name(entry)
    if entry.variation == "base":
        return f"fn {wrapper_name}() -> {_normalize_base_wrapper_return_type(entry.api_signature)}"
    if entry.variation == "hresult":
        return f"fn {wrapper_name}() -> windows::core::HRESULT"
    if entry.variation == "win32_error":
        return f"fn {wrapper_name}() -> windows::Win32::Foundation::WIN32_ERROR"
    if entry.variation == "bool_check":
        return f"fn {wrapper_name}() -> bool"
    return f"fn {wrapper_name}() -> {_normalize_base_wrapper_return_type(entry.api_signature)}"


def _variation_instructions(entry: ManifestEntry) -> str:
    wrapper_name = _wrapper_name(entry)
    if entry.variation == "base":
        return (
            f"Write `{wrapper_name}` so it calls `{entry.api_name}` with reasonable concrete parameter values "
            "and returns a `windows::core::Result<T>` style wrapper result. Use `?` for propagation whenever possible."
        )
    if entry.variation == "hresult":
        return (
            f"Write `{wrapper_name}` so it calls `{entry.api_name}` and returns `windows::core::HRESULT`. "
            "Convert any `Result` error to `HRESULT` with `.map_err(|e| e.code())` or an equivalent direct conversion."
        )
    if entry.variation == "win32_error":
        return (
            f"Write `{wrapper_name}` so it calls `{entry.api_name}` and returns "
            "`windows::Win32::Foundation::WIN32_ERROR`. Convert any `Result` error with "
            "`WIN32_ERROR::from_error(&e)` or an equivalent idiomatic conversion."
        )
    if entry.variation == "bool_check":
        return (
            f"Write `{wrapper_name}` so it calls `{entry.api_name}` and returns `bool`: return `true` on success "
            "and `false` on error, logging the error."
        )
    return f"Write `{wrapper_name}` so it calls `{entry.api_name}` correctly."


def _related_types_block(related_items: List[Dict[str, Any]]) -> str:
    if not related_items:
        return "- (none resolved)"
    lines: List[str] = []
    for item in related_items:
        lines.append(
            "- {name} | {path} | {kind} | `{signature}`".format(
                name=item.get("name", ""),
                path=item.get("path", ""),
                kind=item.get("kind", "unknown"),
                signature=item.get("signature", ""),
            )
        )
    return "\n".join(lines)


def build_problem_prompt(entry: ManifestEntry) -> str:
    return PROBLEM_GENERATION_USER_TEMPLATE.format(
        api_name=entry.api_name,
        api_path=entry.api_path,
        api_signature=entry.api_signature,
        related_types_block=_related_types_block(entry.related_items),
        variation=entry.variation,
        wrapper_name=_wrapper_name(entry),
        wrapper_signature=_wrapper_signature(entry),
        variation_instructions=_variation_instructions(entry),
    )


def _problem_constraints(entry: ManifestEntry) -> List[str]:
    if entry.variation == "base":
        return [
            f"Call `{entry.api_name}` with concrete parameter values",
            f"Handle the result as `{_normalize_base_wrapper_return_type(entry.api_signature)}`",
            "Use `?` for error propagation",
        ]
    if entry.variation == "hresult":
        return [
            f"Call `{entry.api_name}` with concrete parameter values",
            "Return `windows::core::HRESULT` from the wrapper",
            "Convert any API error into `HRESULT` rather than panicking",
        ]
    if entry.variation == "win32_error":
        return [
            f"Call `{entry.api_name}` with concrete parameter values",
            "Return `windows::Win32::Foundation::WIN32_ERROR` from the wrapper",
            "Convert any API error into `WIN32_ERROR` rather than panicking",
        ]
    return [
        f"Call `{entry.api_name}` with concrete parameter values",
        "Return `true` on success and `false` on error",
        "Log the error before returning `false`",
    ]


def _example_line(entry: ManifestEntry) -> str:
    wrapper_name = _wrapper_name(entry)
    if entry.variation == "base":
        return f"let value = {wrapper_name}()?;"
    if entry.variation == "hresult":
        return f"let hr = {wrapper_name}();"
    if entry.variation == "win32_error":
        return f"let err = {wrapper_name}();"
    return f"let ok = {wrapper_name}();"


def build_problem_markdown(entry: ManifestEntry) -> str:
    constraints = "\n".join(f"- {line}" for line in _problem_constraints(entry))
    return (
        f"**Spec:** Write a function `{_wrapper_name(entry)}` that calls `{entry.api_name}` with concrete "
        "parameter values and returns the expected wrapper result.\n\n"
        f"**Constraints:**\n{constraints}\n\n"
        f"**Signature:**\n```rust\n{_wrapper_signature(entry)}\n```\n\n"
        f"**Example:**\n```rust\n{_example_line(entry)}\n```\n"
    )


def _parse_public_fn_declaration(line: str) -> Optional[Tuple[str, str]]:
    match = re.match(
        r'^(?:pub(?:\([^)]*\))?\s+)(?:(?:async|unsafe)\s+)*(?:extern\s+"[^"]+"\s+)?(?:(?:async|unsafe)\s+)*fn\s+([A-Za-z_][A-Za-z0-9_]*)',
        line.strip(),
    )
    if not match:
        return None
    return match.group(1), "function"


def _brace_delta(line: str) -> int:
    delta = 0
    in_string = False
    string_char = ""
    escape = False
    i = 0
    while i < len(line):
        ch = line[i]
        nxt = line[i + 1] if i + 1 < len(line) else ""
        if not in_string and ch == "/" and nxt == "/":
            break
        if ch == "\\" and in_string and not escape:
            escape = True
            i += 1
            continue
        if ch in {'"', "'"} and not escape:
            if in_string and ch == string_char:
                in_string = False
                string_char = ""
            elif not in_string:
                in_string = True
                string_char = ch
            i += 1
            continue
        escape = False
        if not in_string:
            if ch == "{":
                delta += 1
            elif ch == "}":
                delta -= 1
        i += 1
    return delta


def _signature_end_index(text: str) -> Optional[int]:
    paren_depth = 0
    angle_depth = 0
    seen_params = False
    in_string = False
    string_char = ""
    escape = False
    i = 0
    while i < len(text):
        ch = text[i]
        nxt = text[i + 1] if i + 1 < len(text) else ""
        if not in_string and ch == "/" and nxt == "/":
            i += 1
            while i < len(text) and text[i] != "\n":
                i += 1
            i += 1
            continue
        if ch == "\\" and in_string and not escape:
            escape = True
            i += 1
            continue
        if ch in {'"', "'"} and not escape:
            if in_string and ch == string_char:
                in_string = False
                string_char = ""
            elif not in_string:
                in_string = True
                string_char = ch
            i += 1
            continue
        escape = False
        if in_string:
            i += 1
            continue
        if ch == "(":
            seen_params = True
            paren_depth += 1
        elif ch == ")":
            paren_depth = max(0, paren_depth - 1)
        elif ch == "<":
            angle_depth += 1
        elif ch == ">":
            angle_depth = max(0, angle_depth - 1)
        elif ch == ";" and seen_params and paren_depth == 0 and angle_depth == 0:
            return i + 1
        elif ch == "{" and seen_params and paren_depth == 0 and angle_depth == 0:
            return i
        i += 1
    return None


def _scan_signature_from_lines(
    lines: List[str], start_idx: int
) -> Tuple[str, Optional[int], Optional[int], Optional[str]]:
    paren_depth = 0
    angle_depth = 0
    seen_params = False
    in_string = False
    string_char = ""
    escape = False
    pieces: List[str] = []
    started = False
    buffer_length = 0

    for relative_idx, line in enumerate(lines[start_idx:]):
        trimmed_end = line.rstrip()
        if not started and not trimmed_end.strip():
            continue
        if started:
            pieces.append("\n")
            buffer_length += 1
        line_start_offset = buffer_length
        pieces.append(trimmed_end)
        buffer_length += len(trimmed_end)
        started = True

        i = 0
        while i < len(trimmed_end):
            ch = trimmed_end[i]
            nxt = trimmed_end[i + 1] if i + 1 < len(trimmed_end) else ""
            if not in_string and ch == "/" and nxt == "/":
                break
            if ch == "\\" and in_string and not escape:
                escape = True
                i += 1
                continue
            if ch in {'"', "'"} and not escape:
                if in_string and ch == string_char:
                    in_string = False
                    string_char = ""
                elif not in_string:
                    in_string = True
                    string_char = ch
                i += 1
                continue
            escape = False
            if in_string:
                i += 1
                continue
            if ch == "(":
                seen_params = True
                paren_depth += 1
            elif ch == ")":
                paren_depth = max(0, paren_depth - 1)
            elif ch == "<":
                angle_depth += 1
            elif ch == ">":
                angle_depth = max(0, angle_depth - 1)
            elif ch == ";" and seen_params and paren_depth == 0 and angle_depth == 0:
                current = "".join(pieces)
                end_idx = line_start_offset + i + 1
                return current[:end_idx].strip(), relative_idx, i, ";"
            elif ch == "{" and seen_params and paren_depth == 0 and angle_depth == 0:
                current = "".join(pieces)
                end_idx = line_start_offset + i
                return current[:end_idx].strip(), relative_idx, i, "{"
            i += 1

    return "".join(pieces).strip(), None, None, None


def _collect_signature_from_lines(lines: List[str], start_idx: int) -> str:
    signature, _, _, _ = _scan_signature_from_lines(lines, start_idx)
    return signature


def _is_windows_api_wrapper(lines: List[str], start_idx: int) -> bool:
    link_markers = ("windows_core::link!", "windows_core::link !")
    guard_start = max(0, start_idx - 3)
    for idx in range(guard_start, start_idx + 1):
        if any(marker in lines[idx] for marker in link_markers):
            return True

    _, relative_idx, signature_end_col, terminator = _scan_signature_from_lines(lines, start_idx)
    if terminator != "{" or relative_idx is None or signature_end_col is None:
        return False

    brace_depth = 0
    opened_body = False
    body_start_idx = start_idx + relative_idx
    for body_idx in range(body_start_idx, len(lines)):
        body_line = lines[body_idx].rstrip()
        if body_idx == body_start_idx:
            body_line = body_line[signature_end_col:]
        if any(marker in body_line for marker in link_markers):
            return True
        brace_depth += _brace_delta(body_line)
        if "{" in body_line:
            opened_body = True
        if opened_body and brace_depth <= 0:
            return False
    return False


def _module_path_from_file(file_path: Path, src_root: Path) -> str:
    relative = file_path.relative_to(src_root)
    parts = list(relative.parts)
    if not parts:
        return "windows"
    if parts[-1] == "mod.rs":
        parts = parts[:-1]
    else:
        parts[-1] = Path(parts[-1]).stem
    module_parts = ["windows", *parts]
    return "::".join(part for part in module_parts if part)


def _module_path_is_enabled(module_path: str, enabled_features: set[str]) -> bool:
    if module_path == "windows":
        return True
    module_parts = [part for part in module_path.removeprefix("windows::").split("::") if part]
    if not module_parts:
        return True
    for idx in range(len(module_parts), 0, -1):
        candidate = "_".join(module_parts[:idx])
        if candidate in enabled_features:
            return True
    return False


def scan_windows_src(
    src_root: Path,
    enabled_features: Optional[set[str]] = None,
    win32_only: bool = True,
) -> Tuple[List[Dict[str, Any]], ScanSummary]:
    functions: List[Dict[str, Any]] = []
    summary = ScanSummary()
    for file_path in sorted(src_root.rglob("*.rs")):
        normalized_path = str(file_path).replace("\\", "/")
        if win32_only and "/Win32/" not in normalized_path:
            continue
        try:
            lines = file_path.read_text(encoding="utf-8").splitlines()
        except OSError as exc:
            LOGGER.warning("scan_windows_src read_failed file=%s error=%s", file_path, exc)
            continue
        summary.files_scanned += 1
        if summary.files_scanned % 100 == 0:
            LOGGER.info("scan_windows_src progress files_scanned=%s", summary.files_scanned)
        module_path = _module_path_from_file(file_path, src_root)
        for idx, raw_line in enumerate(lines):
            stripped = raw_line.strip()
            parsed = _parse_public_fn_declaration(stripped)
            if parsed is None:
                continue
            summary.pub_fn_declarations += 1
            if not _is_windows_api_wrapper(lines, idx):
                continue
            summary.link_wrappers += 1
            if enabled_features is not None and not _module_path_is_enabled(module_path, enabled_features):
                LOGGER.debug(
                    "scan_windows_src skip_disabled_feature file=%s module_path=%s",
                    file_path,
                    module_path,
                )
                continue
            summary.feature_enabled_wrappers += 1
            name, kind = parsed
            signature = _collect_signature_from_lines(lines, idx)
            functions.append(
                {
                    "name": name,
                    "kind": kind,
                    "file_path": str(file_path),
                    "module_path": module_path,
                    "api_path": f"{module_path}::{name}",
                    "signature": signature,
                }
            )
    functions.sort(key=lambda item: item["api_path"])
    return functions, summary


def _function_group_key(api_path: str) -> str:
    parts = api_path.split("::")
    module_parts = parts[1:-1]
    if not module_parts:
        return api_path
    if len(module_parts) >= 3 and module_parts[0] == "Win32":
        return "_".join(module_parts[:3])
    if len(module_parts) >= 2:
        return "_".join(module_parts[:2])
    return module_parts[0]


def _select_functions_stratified(functions: List[Dict[str, Any]], max_functions: int) -> List[Dict[str, Any]]:
    limit = max(0, max_functions)
    if limit == 0 or not functions:
        return []
    grouped: Dict[str, List[Dict[str, Any]]] = defaultdict(list)
    for function in functions:
        grouped[_function_group_key(function["api_path"])].append(function)
    selected: List[Dict[str, Any]] = []
    offsets = {group_name: 0 for group_name in grouped}
    ordered_groups = sorted(grouped)
    while len(selected) < limit:
        made_progress = False
        for group_name in ordered_groups:
            offset = offsets[group_name]
            bucket = grouped[group_name]
            if offset >= len(bucket):
                continue
            selected.append(bucket[offset])
            offsets[group_name] = offset + 1
            made_progress = True
            if len(selected) >= limit:
                break
        if not made_progress:
            break
    return selected


def _extract_related_names(signature: str) -> List[str]:
    function_name = ""
    fn_match = re.search(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)", signature)
    if fn_match:
        function_name = fn_match.group(1)

    ignore = {
        function_name,
        "Result",
        "Error",
        "Option",
        "Some",
        "None",
        "Ok",
        "Err",
        "String",
        "Vec",
        "Self",
        "HRESULT",
        "Windows",
        "Win32",
    }

    names: set[str] = set()
    for match in re.findall(r"\b([A-Z][A-Za-z0-9_]+|[A-Z0-9_]{2,})\b", signature):
        if match not in ignore:
            names.add(match)

    for match in re.findall(r"(?:[A-Za-z_][A-Za-z0-9_]*::)+([A-Za-z_][A-Za-z0-9_]*)", signature):
        if match not in ignore and match[:1].isupper():
            names.add(match)

    return sorted(names)


def _search_rustdoc(name: str, rustdocs_base: str, client: httpx.Client, limit: int = 3) -> List[Dict[str, Any]]:
    response = client.get(
        f"{rustdocs_base.rstrip('/')}/search",
        params={"q": name, "limit": limit},
    )
    response.raise_for_status()
    payload = response.json()
    if isinstance(payload, dict):
        results = payload.get("results")
        return results if isinstance(results, list) else []
    if isinstance(payload, list):
        return payload
    return []


def resolve_related_items(
    signature: str,
    src_root: Path,
    rustdocs_base: str,
    client: httpx.Client,
) -> List[Dict[str, Any]]:
    del src_root
    related_items: List[Dict[str, Any]] = []
    seen_paths: set[str] = set()
    for name in _extract_related_names(signature):
        try:
            results = _search_rustdoc(name, rustdocs_base, client, limit=3)
        except Exception as exc:
            LOGGER.warning("resolve_related_items lookup_failed symbol=%s error=%s", name, exc)
            continue
        for item in results:
            if not isinstance(item, dict):
                continue
            path = item.get("path") or item.get("full_path") or item.get("name") or ""
            if not isinstance(path, str) or not path or path in seen_paths:
                continue
            seen_paths.add(path)
            related_items.append(
                {
                    "name": name,
                    "path": path,
                    "kind": item.get("kind") or item.get("type") or "unknown",
                    "signature": item.get("signature") or item.get("decl") or "",
                }
            )
            break
    return related_items


def _manifest_path(output_dir: Path) -> Path:
    return output_dir / "manifest.jsonl"


def _load_manifest_entries(manifest_path: Path) -> List[ManifestEntry]:
    entries: List[ManifestEntry] = []
    if not manifest_path.exists():
        return entries
    with manifest_path.open("r", encoding="utf-8") as handle:
        for raw_line in handle:
            line = raw_line.strip()
            if not line:
                continue
            try:
                payload = json.loads(line)
            except json.JSONDecodeError:
                LOGGER.warning("load_manifest_entries invalid_json path=%s", manifest_path)
                continue
            if not isinstance(payload, dict):
                continue
            entries.append(
                ManifestEntry(
                    id=str(payload.get("id") or uuid.uuid4()),
                    api_name=str(payload.get("api_name") or ""),
                    api_path=str(payload.get("api_path") or ""),
                    api_signature=str(payload.get("api_signature") or ""),
                    related_items=payload.get("related_items") if isinstance(payload.get("related_items"), list) else [],
                    variation=str(payload.get("variation") or "base"),
                    status=str(payload.get("status") or "pending"),
                    problem_id=payload.get("problem_id") if isinstance(payload.get("problem_id"), str) else None,
                )
            )
    return entries


def _write_manifest_entries(manifest_path: Path, entries: List[ManifestEntry]) -> None:
    temp_path = manifest_path.with_suffix(".jsonl.tmp")
    with temp_path.open("w", encoding="utf-8") as handle:
        for entry in entries:
            handle.write(json.dumps(asdict(entry), ensure_ascii=False) + "\n")
    temp_path.replace(manifest_path)


def extract_symbols_from_rust_folder(folder: Path) -> set[str]:
    ignore = {
        "Result",
        "Error",
        "Option",
        "Some",
        "None",
        "Ok",
        "Err",
        "String",
        "Vec",
        "Self",
        "HRESULT",
        "Windows",
        "Win32",
    }
    symbols: set[str] = set()
    for file_path in folder.rglob("*.rs"):
        try:
            source = file_path.read_text(encoding="utf-8")
        except Exception as exc:
            LOGGER.warning("extract_symbols_from_rust_folder read_failed file=%s error=%s", file_path, exc)
            continue
        for match in re.findall(r"\b([A-Z][A-Za-z0-9_]+|[A-Z0-9_]{2,})\b", source):
            if match not in ignore:
                symbols.add(match)
    LOGGER.info("extract_symbols_from_rust_folder folder=%s symbols_found=%s", folder, len(symbols))
    return symbols


def generate_manifest(
    src_root: Path,
    output_dir: Path,
    rustdocs_base: str,
    client: httpx.Client,
    max_functions: int,
    manifest_workers: int,
    variations: List[str],
    priority_symbols: Optional[set[str]] = None,
    win32_only: bool = True,
    overwrite: bool = False,
) -> Path:
    output_dir.mkdir(parents=True, exist_ok=True)
    manifest_path = _manifest_path(output_dir)
    existing_entries: List[ManifestEntry] = []
    existing_api_paths: set[str] = set()
    selection_limit = max_functions
    mode = "full"
    if manifest_path.exists() and not overwrite:
        existing_entries = _load_manifest_entries(manifest_path)
        existing_api_paths = {entry.api_path for entry in existing_entries if entry.api_path}
        existing_function_count = len(existing_api_paths)
        if existing_function_count >= max_functions:
            LOGGER.info(
                "generate_manifest reuse_existing path=%s existing_functions=%s max_functions=%s",
                manifest_path,
                existing_function_count,
                max_functions,
            )
            return manifest_path
        selection_limit = max_functions - existing_function_count
        mode = "topup"

    functions, scan_summary = scan_windows_src(
        src_root,
        enabled_features=ENABLED_WINDOWS_FEATURES,
        win32_only=win32_only,
    )
    candidate_functions = functions
    if mode == "topup":
        candidate_functions = [
            function for function in functions if function["api_path"] not in existing_api_paths
        ]
    if priority_symbols:
        priority_functions = [
            function for function in candidate_functions if function["name"] in priority_symbols
        ]
        remaining_functions = [
            function for function in candidate_functions if function["name"] not in priority_symbols
        ]
        candidate_functions = priority_functions + remaining_functions
    selected_functions = _select_functions_stratified(candidate_functions, selection_limit)
    if mode == "topup" and not selected_functions:
        LOGGER.warning(
            "generate_manifest no_new_candidates path=%s existing_functions=%s max_functions=%s",
            manifest_path,
            len(existing_api_paths),
            max_functions,
        )
        return manifest_path
    LOGGER.info(
        "generate_manifest scan_summary files_scanned=%s pub_fn_declarations=%s link_wrappers=%s feature_enabled_wrappers=%s selected_functions=%s mode=%s",
        scan_summary.files_scanned,
        scan_summary.pub_fn_declarations,
        scan_summary.link_wrappers,
        scan_summary.feature_enabled_wrappers,
        len(selected_functions),
        mode,
    )
    new_entries: List[ManifestEntry] = []
    related_items_by_index: List[List[Dict[str, Any]]] = [[] for _ in selected_functions]

    with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, manifest_workers)) as executor:
        future_to_index = {
            executor.submit(
                resolve_related_items,
                signature=function["signature"],
                src_root=src_root,
                rustdocs_base=rustdocs_base,
                client=client,
            ): idx
            for idx, function in enumerate(selected_functions)
        }
        for future in concurrent.futures.as_completed(future_to_index):
            idx = future_to_index[future]
            try:
                related_items_by_index[idx] = future.result()
            except Exception as exc:
                LOGGER.warning(
                    "generate_manifest resolve_related_items_failed api=%s error=%s",
                    selected_functions[idx]["api_path"],
                    exc,
                )
                related_items_by_index[idx] = []

    for function, related_items in zip(selected_functions, related_items_by_index):
        for variation in variations:
            new_entries.append(
                ManifestEntry(
                    id=str(uuid.uuid4()),
                    api_name=function["name"],
                    api_path=function["api_path"],
                    api_signature=function["signature"],
                    related_items=related_items,
                    variation=variation,
                    status="pending",
                    problem_id=None,
                )
            )

    manifest_entries = existing_entries + new_entries
    _write_manifest_entries(manifest_path, manifest_entries)
    LOGGER.info(
        "generate_manifest wrote path=%s mode=%s existing_entries=%s new_functions=%s new_entries=%s entries=%s",
        manifest_path,
        mode,
        len(existing_entries),
        len(selected_functions),
        len(new_entries),
        len(manifest_entries),
    )
    return manifest_path


def generate_one_problem(
    entry: ManifestEntry,
    max_repair_attempts: int,
    eval_base: str,
    rustdocs_base: str,
    client: httpx.Client,
    recorder: StepRecorder,
    run_id: str,
) -> Optional[GeneratedProblem]:
    problem_md = build_problem_markdown(entry)
    user_prompt = build_problem_prompt(entry)

    best_code = ""
    best_eval: Dict[str, Any] = {}
    best_score = 10**9
    previous_code = ""
    same_streak = 0
    repair_context = ""

    for attempt in range(1, max_repair_attempts + 1):
        prompt = user_prompt if attempt == 1 else REPAIR_PROMPT_TEMPLATE.format(context=repair_context)
        messages = [
            {"role": "system", "content": SOLUTION_SYSTEM_PROMPT},
            {"role": "user", "content": prompt},
        ]

        response_text: Optional[str] = None
        for retry in range(2):
            response_text = openrouter_generate_code(messages)
            if response_text is not None:
                break
            if retry == 0:
                retry_prompt = prompt + "\n\nPlease generate code. Output only a ```rust code block."
                messages = [
                    {"role": "system", "content": SOLUTION_SYSTEM_PROMPT},
                    {"role": "user", "content": retry_prompt},
                ]

        if response_text is None:
            LOGGER.warning(
                "generate_one_problem solution_generation_failed run_id=%s attempt=%s api=%s variation=%s",
                run_id,
                attempt,
                entry.api_name,
                entry.variation,
            )
            continue

        code = extract_rust_code_block(response_text)
        if code is None:
            recorder.record_step(
                attempt=attempt,
                step_type="no_code",
                code="",
                eval_result=None,
                extra_context={
                    "api_name": entry.api_name,
                    "variation": entry.variation,
                    "response_preview": preview_text(response_text, limit=500),
                },
            )
            repair_context = (
                "## Build/Test Results\n"
                "No Rust code block was generated in the previous attempt.\n\n"
                "## Repair Instructions\n"
                "- Output the complete src/main.rs in a single ```rust code fence.\n"
            )
            continue

        recorder.record_step(
            attempt=attempt,
            step_type="generate",
            code=code,
            eval_result=None,
            extra_context={
                "api_name": entry.api_name,
                "variation": entry.variation,
                "phase": "initial" if attempt == 1 else "repair",
            },
        )

        symbols = extract_windows_api_symbols(code)
        rustdoc_info = ""
        try:
            rustdoc_info = batch_rustdoc_lookup(symbols, rustdocs_base, client)
        except Exception as exc:
            LOGGER.warning("generate_one_problem rustdoc_lookup_failed run_id=%s attempt=%s error=%s", run_id, attempt, exc)

        eval_result: Dict[str, Any] = {}
        eval_error: Optional[Exception] = None
        for eval_try in range(2):
            try:
                eval_result = eval_server_evaluate(
                    main_rs=code,
                    unit_tests_private="",
                    fixed_deps=FIXED_DEPENDENCIES,
                    eval_base=eval_base,
                    client=client,
                    run_tests=False,
                )
                eval_error = None
                break
            except (httpx.TimeoutException, httpx.HTTPStatusError) as exc:
                eval_error = exc
                LOGGER.warning(
                    "generate_one_problem eval_retry run_id=%s attempt=%s eval_try=%s error=%s",
                    run_id,
                    attempt,
                    eval_try + 1,
                    exc,
                )
            except Exception as exc:
                eval_error = exc
                LOGGER.warning(
                    "generate_one_problem eval_failed run_id=%s attempt=%s error=%s",
                    run_id,
                    attempt,
                    exc,
                )
                break

        if eval_error is not None and not eval_result:
            recorder.record_step(
                attempt=attempt,
                step_type="eval_error",
                code=code,
                eval_result=None,
                extra_context={"api_name": entry.api_name, "variation": entry.variation, "error": str(eval_error)},
            )
            repair_context = (
                "## Build/Test Results\n"
                f"Evaluator request failed: {eval_error}\n\n"
                "## Repair Instructions\n"
                "- Keep the same approach and output valid Rust in a single fence.\n"
            )
            continue

        recorder.record_step(
            attempt=attempt,
            step_type="eval",
            code=code,
            eval_result=eval_result,
            extra_context={"api_name": entry.api_name, "variation": entry.variation, "symbols": symbols, "rustdoc_info": rustdoc_info},
        )

        if _is_feature_not_enabled_error(eval_result):
            LOGGER.warning(
                "generate_one_problem feature_not_enabled_abort run_id=%s api=%s variation=%s",
                run_id,
                entry.api_name,
                entry.variation,
            )
            return None

        score = _error_score(eval_result)
        if score < best_score:
            best_score = score
            best_code = code
            best_eval = eval_result

        if eval_result.get("ok") is True:
            formatted = None
            try:
                formatted = eval_server_format(code, eval_base, client)
            except Exception as exc:
                LOGGER.warning("generate_one_problem format_failed run_id=%s attempt=%s error=%s", run_id, attempt, exc)
                formatted = None

            if formatted and formatted.strip() != code.strip():
                recorder.record_step(
                    attempt=attempt,
                    step_type="format",
                    code=formatted,
                    eval_result=None,
                    extra_context={"api_name": entry.api_name, "variation": entry.variation},
                )
                try:
                    formatted_eval = eval_server_evaluate(
                        main_rs=formatted,
                        unit_tests_private="",
                        fixed_deps=FIXED_DEPENDENCIES,
                        eval_base=eval_base,
                        client=client,
                        run_tests=False,
                    )
                except Exception as exc:
                    LOGGER.warning(
                        "generate_one_problem formatted_recheck_failed run_id=%s attempt=%s error=%s",
                        run_id,
                        attempt,
                        exc,
                    )
                    formatted_eval = None

                if isinstance(formatted_eval, dict) and formatted_eval.get("ok") is True:
                    recorder.record_final(formatted, formatted_eval)
                    return GeneratedProblem(
                        entry_id=entry.id,
                        problem_md=problem_md,
                        main_rs=formatted.rstrip() + "\n",
                        last_eval=formatted_eval,
                    )

            recorder.record_final(code, eval_result)
            return GeneratedProblem(
                entry_id=entry.id,
                problem_md=problem_md,
                main_rs=code.rstrip() + "\n",
                last_eval=eval_result,
            )

        diagnostic_symbols = extract_symbols_from_diagnostics(eval_result)
        targeted_info = ""
        if diagnostic_symbols:
            try:
                targeted_info = batch_rustdoc_lookup(diagnostic_symbols, rustdocs_base, client)
            except Exception as exc:
                LOGGER.warning(
                    "generate_one_problem targeted_lookup_failed run_id=%s attempt=%s error=%s",
                    run_id,
                    attempt,
                    exc,
                )

        combined_info_parts = [part for part in [rustdoc_info, targeted_info] if part.strip()]
        combined_info = "\n\n".join(combined_info_parts)
        repair_context = build_repair_context(
            eval_result=eval_result,
            main_rs=code,
            rustdoc_info=combined_info,
            problem_text=problem_md,
        )

        if code.strip() == previous_code.strip():
            same_streak += 1
        else:
            same_streak = 0
        previous_code = code

        if same_streak >= 2:
            hint = _first_diagnostic_hint(eval_result)
            repair_context += (
                "\n\nWARNING: Your previous repair attempt returned identical code. "
                "You MUST make a different change this time. Focus on: "
                f"{hint}"
            )

    if best_code.strip():
        LOGGER.warning(
            "generate_one_problem exhausted_attempts returning_best run_id=%s api=%s variation=%s best_score=%s",
            run_id,
            entry.api_name,
            entry.variation,
            best_score,
        )
        recorder.record_step(
            attempt=max_repair_attempts + 1,
            step_type="best_effort",
            code=best_code,
            eval_result=best_eval,
            extra_context={"api_name": entry.api_name, "variation": entry.variation, "best_score": best_score},
        )
        return GeneratedProblem(
            entry_id=entry.id,
            problem_md=problem_md,
            main_rs=best_code.rstrip() + "\n",
            last_eval=best_eval,
            verified=False,
        )

    return None


def _write_problem_artifacts(output_dir: Path, problem_id: str, generated: GeneratedProblem) -> None:
    problems_dir = output_dir / "problems"
    solutions_dir = output_dir / "solutions"
    problems_dir.mkdir(parents=True, exist_ok=True)
    solutions_dir.mkdir(parents=True, exist_ok=True)
    (problems_dir / f"{problem_id}.md").write_text(generated.problem_md, encoding="utf-8")
    (solutions_dir / f"{problem_id}.rs").write_text(generated.main_rs, encoding="utf-8")


def generate_problems_from_manifest(
    manifest_path: Path,
    output_dir: Path,
    max_repair_attempts: int,
    eval_base: str,
    rustdocs_base: str,
    client: httpx.Client,
    workers: int = 1,
    overwrite: bool = False,
) -> None:
    entries = _load_manifest_entries(manifest_path)
    if not entries:
        LOGGER.info("generate_problems_from_manifest no_entries manifest=%s", manifest_path)
        return

    output_dir.mkdir(parents=True, exist_ok=True)
    (output_dir / "problems").mkdir(parents=True, exist_ok=True)
    (output_dir / "solutions").mkdir(parents=True, exist_ok=True)
    (output_dir / "steps").mkdir(parents=True, exist_ok=True)

    manifest_lock = threading.Lock()
    index_by_id = {entry.id: idx for idx, entry in enumerate(entries)}
    eval_server_warmup(eval_base, client)

    pending_entries = [entry for entry in entries if overwrite or entry.status != "ok"]
    if not pending_entries:
        LOGGER.info("generate_problems_from_manifest nothing_to_do manifest=%s", manifest_path)
        return

    def persist_update(updated_entry: ManifestEntry) -> None:
        with manifest_lock:
            entries[index_by_id[updated_entry.id]] = updated_entry
            _write_manifest_entries(manifest_path, entries)

    def worker(entry: ManifestEntry) -> Tuple[ManifestEntry, Optional[GeneratedProblem], Optional[Exception]]:
        run_id = uuid.uuid4().hex[:8]
        recorder = StepRecorder(run_id=run_id, output_dir=output_dir)
        try:
            with httpx.Client(timeout=120.0) as local_client:
                generated = generate_one_problem(
                    entry=entry,
                    max_repair_attempts=max_repair_attempts,
                    eval_base=eval_base,
                    rustdocs_base=rustdocs_base,
                    client=local_client,
                    recorder=recorder,
                    run_id=run_id,
                )
            return entry, generated, None
        except Exception as exc:
            return entry, None, exc

    max_workers = max(1, int(workers))
    with concurrent.futures.ThreadPoolExecutor(max_workers=max_workers) as executor:
        future_map: Dict[concurrent.futures.Future[Tuple[ManifestEntry, Optional[GeneratedProblem], Optional[Exception]]], ManifestEntry] = {}
        for entry in pending_entries:
            future = executor.submit(worker, entry)
            future_map[future] = entry

        for future in concurrent.futures.as_completed(future_map):
            original_entry = future_map[future]
            try:
                entry, generated, error = future.result()
            except Exception as exc:
                updated_entry = ManifestEntry(
                    id=original_entry.id,
                    api_name=original_entry.api_name,
                    api_path=original_entry.api_path,
                    api_signature=original_entry.api_signature,
                    related_items=original_entry.related_items,
                    variation=original_entry.variation,
                    status="failed",
                    problem_id=original_entry.problem_id,
                )
                persist_update(updated_entry)
                LOGGER.exception("generate_problems_from_manifest worker_failed entry_id=%s error=%s", original_entry.id, exc)
                continue

            if error is not None:
                updated_entry = ManifestEntry(
                    id=entry.id,
                    api_name=entry.api_name,
                    api_path=entry.api_path,
                    api_signature=entry.api_signature,
                    related_items=entry.related_items,
                    variation=entry.variation,
                    status="failed",
                    problem_id=entry.problem_id,
                )
                persist_update(updated_entry)
                LOGGER.exception("generate_problems_from_manifest entry_failed entry_id=%s error=%s", entry.id, error)
                continue

            if generated is not None and generated.verified:
                problem_id = str(uuid.uuid4())
                _write_problem_artifacts(output_dir, problem_id, generated)
                updated_entry = ManifestEntry(
                    id=entry.id,
                    api_name=entry.api_name,
                    api_path=entry.api_path,
                    api_signature=entry.api_signature,
                    related_items=entry.related_items,
                    variation=entry.variation,
                    status="ok",
                    problem_id=problem_id,
                )
                persist_update(updated_entry)
                LOGGER.info(
                    "generate_problems_from_manifest ok entry_id=%s api=%s variation=%s problem_id=%s",
                    entry.id,
                    entry.api_name,
                    entry.variation,
                    problem_id,
                )
                continue

            updated_entry = ManifestEntry(
                id=entry.id,
                api_name=entry.api_name,
                api_path=entry.api_path,
                api_signature=entry.api_signature,
                related_items=entry.related_items,
                variation=entry.variation,
                status="failed",
                problem_id=entry.problem_id,
            )
            persist_update(updated_entry)
            LOGGER.warning(
                "generate_problems_from_manifest failed entry_id=%s api=%s variation=%s",
                entry.id,
                entry.api_name,
                entry.variation,
            )


if __name__ == "__main__":
    configure_logging()

    parser = argparse.ArgumentParser(description="Generate Windows API usage problems and verified solutions.")
    parser.add_argument(
        "--src-root",
        default="../rustdoc-search/data/windows/src/Windows",
        help="Path to the windows crate source tree.",
    )
    parser.add_argument(
        "--output-dir",
        default="./windows_api_out",
        help="Root output directory for manifest, problems, solutions, and steps.",
    )
    parser.add_argument(
        "--max-functions",
        type=int,
        default=200,
        help="Maximum number of Windows API functions to include in the manifest.",
    )
    parser.add_argument(
        "--variations",
        nargs="+",
        default=["base", "hresult", "win32_error"],
        help="Variation types to generate per function.",
    )
    parser.add_argument(
        "--max-attempts",
        type=int,
        default=8,
        help="Max repair attempts per generated solution.",
    )
    parser.add_argument(
        "--workers",
        type=int,
        default=4,
        help="Maximum number of concurrent generation workers.",
    )
    parser.add_argument(
        "--manifest-workers",
        type=int,
        default=16,
        help="Number of parallel workers for resolving rustdoc symbols during manifest generation.",
    )
    parser.add_argument(
        "--overwrite",
        action="store_true",
        help="Re-generate even if the manifest or outputs already exist.",
    )
    parser.add_argument(
        "--win32-only",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="Restrict manifest scanning to the Windows/Win32 source tree.",
    )
    parser.add_argument(
        "--skip-manifest",
        action="store_true",
        help="Skip manifest regeneration when an existing manifest is already present.",
    )
    parser.add_argument(
        "--priority-src",
        default=None,
        help="Path to a folder of Rust files whose Windows symbols are used as priority targets in the manifest.",
    )
    args = parser.parse_args()

    src_root = Path(args.src_root)
    output_dir = Path(args.output_dir)
    manifest_path = _manifest_path(output_dir)
    eval_base = env("RUST_EVAL_BASE_URL", "http://127.0.0.1:3002")
    rustdocs_base = env("RUSTDOCS_BASE_URL", "http://127.0.0.1:3001")
    priority_src = Path(args.priority_src).resolve() if args.priority_src is not None else None
    priority_symbols = extract_symbols_from_rust_folder(priority_src) if priority_src is not None else set()

    LOGGER.info(
        "windows_api_usage_agent start src_root=%s output_dir=%s max_functions=%s variations=%s max_attempts=%s workers=%s manifest_workers=%s overwrite=%s win32_only=%s skip_manifest=%s priority_src=%s priority_symbols_count=%s cwd=%s",
        src_root,
        output_dir,
        args.max_functions,
        args.variations,
        args.max_attempts,
        args.workers,
        args.manifest_workers,
        args.overwrite,
        args.win32_only,
        args.skip_manifest,
        priority_src,
        len(priority_symbols),
        os.getcwd(),
    )

    with httpx.Client(timeout=120.0) as client:
        should_generate_manifest = True
        if args.skip_manifest and manifest_path.exists() and not args.overwrite:
            should_generate_manifest = False

        if should_generate_manifest:
            manifest_path = generate_manifest(
                src_root=src_root,
                output_dir=output_dir,
                rustdocs_base=rustdocs_base,
                client=client,
                max_functions=args.max_functions,
                manifest_workers=args.manifest_workers,
                variations=list(args.variations),
                priority_symbols=priority_symbols,
                win32_only=args.win32_only,
                overwrite=args.overwrite,
            )

        generate_problems_from_manifest(
            manifest_path=manifest_path,
            output_dir=output_dir,
            max_repair_attempts=args.max_attempts,
            eval_base=eval_base,
            rustdocs_base=rustdocs_base,
            client=client,
            workers=args.workers,
            overwrite=args.overwrite,
        )
