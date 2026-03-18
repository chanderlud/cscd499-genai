import argparse
import concurrent.futures
import json
import os
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, List, Optional, Set

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


FIXED_DEPENDENCIES = (
    Path(__file__).resolve().parent.parent / "rust_dependencies.md"
).read_text(encoding="utf-8")

IDEATION_SYSTEM_PROMPT = """You are a technical problem-statement author for Rust/Win32 coding challenges.

Given a Windows API snippet, propose a NEW coding challenge idea inspired by the same API area, then provide a concise, self-contained Markdown problem description.

Required output structure:
- A `TITLE:` line with a short imperative title.
- A `PROBLEM:` sentinel on its own line.
- A Markdown block with:
  - `**Spec:**` bullet describing what the function must do
  - `**Constraints:**` bullet listing key requirements
  - `**Signature:**` bullet with a fenced Rust signature block
  - `**Example:**` bullet with a minimal usage snippet

Signature & Example rules:
- Use `windows::core::Result<T>` (or `windows::core::Result<()>`) for all fallible return types in the signature.
- Do NOT include any `use windows::...` import lines in the Signature or Example blocks.
- The Example block should show a call-site usage only, not import declarations.

Do not include any extra prose outside this structure.
"""

IDEATION_USER_TEMPLATE = """Invent one NEW Rust/Win32 challenge inspired by this snippet.

Rules:
- Reuse the same Windows API surface area/theme.
- The new task must be meaningfully different from the source snippet.
- Do not copy the original behavior directly.
- Keep the problem self-contained and concise.

Already generated ideas:
{previously_generated}

If there are no more meaningfully different ideas left for this snippet, respond with exactly:
NO_MORE_IDEAS

## Source snippet
```rust
{snippet_code}
```
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

// ❌ WRONG — Error has no from_win32(code) method:
//   windows::core::Error::from_win32(code)
//
// ✅ CORRECT — use HRESULT::from_win32 for a code, Error::from_thread() for GetLastError:
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

REALIGN_SYSTEM_PROMPT = """You are a problem-statement editor for Rust/Win32 coding challenges.

Your sole job is to update the problem description so it matches the accepted solution exactly, without changing the challenge's intent.

Rules:
- Preserve the original `TITLE:` and `PROBLEM:` sentinel structure exactly.
- Update `**Spec:**`, `**Constraints:**`, `**Signature:**`, and `**Example:**` so they accurately reflect what the solution actually implements.
- The `**Signature:**` must use `windows::core::Result<T>` (or `windows::core::Result<()>`) for any fallible function.
- Do NOT include any `use windows::...` import lines in the `**Signature:**` or `**Example:**` blocks.
- Do not add new requirements that the solution does not satisfy.
- Do not remove requirements that the solution does satisfy.
- Output only the updated problem block starting at `TITLE:` through the end of the Markdown block.
- Do not output any additional prose.
"""

REALIGN_USER_TEMPLATE = """## Original Problem
{problem_md}

## Accepted Solution
```rust
{solution_code}
```

Re-align the problem description to match the solution exactly, following the system instructions.
"""


@dataclass
class ProblemSolutionResult:
    idea: str
    problem_md: str
    main_rs: str
    last_eval: Dict[str, Any]
    verified: bool = True


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


def _extract_title(response_text: str) -> str:
    for raw_line in response_text.splitlines():
        line = raw_line.strip()
        if line.startswith("TITLE:"):
            title = line[len("TITLE:") :].strip()
            if title:
                return title
    return "Write a Windows API utility"


def _extract_problem_md(response_text: str) -> Optional[str]:
    stripped = response_text.strip()
    if not stripped:
        return None

    lines = stripped.splitlines()
    sentinel_index = None
    for index, line in enumerate(lines):
        if line.strip() == "PROBLEM:":
            sentinel_index = index
            break

    if sentinel_index is None:
        problem_md = stripped
    else:
        problem_md = "\n".join(lines[sentinel_index + 1 :]).strip()

    return problem_md or None


def _realign_problem(problem_md: str, solution_code: str, run_id: str) -> str:
    messages = [
        {"role": "system", "content": REALIGN_SYSTEM_PROMPT},
        {
            "role": "user",
            "content": REALIGN_USER_TEMPLATE.format(problem_md=problem_md, solution_code=solution_code),
        },
    ]
    response_text = openrouter_generate_code(messages)
    if response_text is None or not response_text.strip():
        LOGGER.warning("realign_problem empty_response run_id=%s", run_id)
        return problem_md

    aligned_problem_md = _extract_problem_md(response_text)
    if not aligned_problem_md:
        LOGGER.warning("realign_problem invalid_problem_md run_id=%s", run_id)
        return problem_md
    return aligned_problem_md


def _format_previous_ideas(previously_generated: List[ProblemSolutionResult]) -> str:
    if not previously_generated:
        return "(none yet)"
    lines: List[str] = []
    for idx, item in enumerate(previously_generated, start=1):
        symbols = extract_windows_api_symbols(item.main_rs)
        brief = ", ".join(symbols[:5]) if symbols else "no symbol summary available"
        lines.append(f"{idx}. {item.idea} - {brief}")
    return "\n".join(lines)


def load_produced_ideas(output_root: Path, include_failed: bool = False) -> Set[str]:
    ideas: Set[str] = set()
    manifests = list(output_root.rglob("manifest.jsonl"))
    for manifest_path in manifests:
        count_before = len(ideas)
        try:
            with manifest_path.open("r", encoding="utf-8") as handle:
                for line in handle:
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        record = json.loads(line)
                    except json.JSONDecodeError:
                        continue
                    idea = record.get("idea")
                    if not isinstance(idea, str) or not idea.strip():
                        continue
                    if include_failed or record.get("ok") is True:
                        ideas.add(idea.strip())
            per_file = len(ideas) - count_before
            LOGGER.info("load_produced_ideas file=%s ideas_loaded=%s", manifest_path, per_file)
        except OSError as exc:
            LOGGER.warning("load_produced_ideas skip file=%s error=%s", manifest_path, exc)
    LOGGER.info("load_produced_ideas total manifests=%s total_ideas=%s", len(manifests), len(ideas))
    return ideas


def _collect_input_samples(input_path: Path) -> List[Path]:
    if input_path.is_file() and input_path.suffix.lower() == ".rs":
        return [input_path]
    if input_path.is_dir():
        return sorted(path for path in input_path.rglob("*.rs") if path.is_file())
    raise ValueError(f"Input must be a .rs file or directory of .rs files: {input_path}")


def generate_one_problem(
    snippet_code: str,
    previously_generated: List[ProblemSolutionResult],
    max_repair_attempts: int,
    eval_base: str,
    rustdocs_base: str,
    client: httpx.Client,
    recorder: StepRecorder,
    run_id: str,
) -> Optional[ProblemSolutionResult]:
    previous_list = _format_previous_ideas(previously_generated)
    ideation_prompt = IDEATION_USER_TEMPLATE.format(
        snippet_code=snippet_code,
        previously_generated=previous_list,
    )
    messages = [
        {"role": "system", "content": IDEATION_SYSTEM_PROMPT},
        {"role": "user", "content": ideation_prompt},
    ]

    ideation_response: Optional[str] = None
    for retry in range(2):
        ideation_response = openrouter_generate_code(messages)
        if ideation_response is not None:
            break
        if retry == 0:
            retry_prompt = (
                ideation_prompt
                + "\n\nPlease output `TITLE:`, then `PROBLEM:`, then the Markdown block exactly in the required structure."
            )
            messages = [
                {"role": "system", "content": IDEATION_SYSTEM_PROMPT},
                {"role": "user", "content": retry_prompt},
            ]

    if ideation_response is None:
        LOGGER.warning("generate_one_problem ideation_failed run_id=%s", run_id)
        return None

    stripped_response = ideation_response.strip()
    if "NO_MORE_IDEAS" in stripped_response:
        recorder.record_step(
            attempt=1,
            step_type="ideate_opt_out",
            code="",
            eval_result=None,
            extra_context={"response": preview_text(stripped_response, limit=300)},
        )
        return None

    idea = _extract_title(ideation_response)
    problem_md = _extract_problem_md(ideation_response)
    if not problem_md:
        recorder.record_step(
            attempt=1,
            step_type="ideate_generate",
            code="",
            eval_result=None,
            extra_context={"idea": idea, "response_preview": preview_text(ideation_response, limit=500)},
        )
        LOGGER.warning("generate_one_problem empty_problem_md run_id=%s idea=%r", run_id, idea)
        return None

    recorder.record_step(
        attempt=1,
        step_type="ideate_generate",
        code="",
        eval_result=None,
        extra_context={"idea": idea, "problem_preview": preview_text(problem_md, limit=500)},
    )

    best_code = ""
    best_eval: Dict[str, Any] = {}
    best_score = 10**9
    previous_code = ""
    same_streak = 0
    repair_context = ""

    for attempt in range(1, max_repair_attempts + 1):
        user_prompt = problem_md if attempt == 1 else REPAIR_PROMPT_TEMPLATE.format(context=repair_context)
        solve_messages = [
            {"role": "system", "content": SOLUTION_SYSTEM_PROMPT},
            {"role": "user", "content": user_prompt},
        ]

        response_text: Optional[str] = None
        for retry in range(2):
            response_text = openrouter_generate_code(solve_messages)
            if response_text is not None:
                break
            if retry == 0:
                retry_prompt = user_prompt + "\n\nPlease generate code. Output only a ```rust code block."
                solve_messages = [
                    {"role": "system", "content": SOLUTION_SYSTEM_PROMPT},
                    {"role": "user", "content": retry_prompt},
                ]

        if response_text is None:
            LOGGER.warning(
                "generate_one_problem solution_generation_failed run_id=%s attempt=%s idea=%r",
                run_id,
                attempt,
                idea,
            )
            continue

        code = extract_rust_code_block(response_text)
        if code is None:
            recorder.record_step(
                attempt=attempt,
                step_type="no_code",
                code="",
                eval_result=None,
                extra_context={"idea": idea, "response_preview": preview_text(response_text, limit=500)},
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
            extra_context={"idea": idea, "phase": "initial" if attempt == 1 else "repair"},
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
                extra_context={"idea": idea, "error": str(eval_error)},
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
            extra_context={"idea": idea, "symbols": symbols, "rustdoc_info": rustdoc_info},
        )

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
                    extra_context={"idea": idea},
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
                    aligned_problem_md = _realign_problem(problem_md, formatted.rstrip() + "\n", run_id)
                    recorder.record_step(
                        attempt=attempt,
                        step_type="realign",
                        code=formatted.rstrip() + "\n",
                        eval_result=None,
                        extra_context={
                            "idea": idea,
                            "original_problem_preview": preview_text(problem_md, 300),
                            "aligned_problem_preview": preview_text(aligned_problem_md, 300),
                        },
                    )
                    return ProblemSolutionResult(
                        idea=idea,
                        problem_md=aligned_problem_md,
                        main_rs=formatted.rstrip() + "\n",
                        last_eval=formatted_eval,
                    )

            aligned_problem_md = _realign_problem(problem_md, code.rstrip() + "\n", run_id)
            recorder.record_step(
                attempt=attempt,
                step_type="realign",
                code=code.rstrip() + "\n",
                eval_result=None,
                extra_context={
                    "idea": idea,
                    "original_problem_preview": preview_text(problem_md, 300),
                    "aligned_problem_preview": preview_text(aligned_problem_md, 300),
                },
            )
            return ProblemSolutionResult(
                idea=idea,
                problem_md=aligned_problem_md,
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
            "generate_one_problem exhausted_attempts returning_best run_id=%s idea=%r best_score=%s",
            run_id,
            idea,
            best_score,
        )
        recorder.record_step(
            attempt=max_repair_attempts + 1,
            step_type="best_effort",
            code=best_code,
            eval_result=best_eval,
            extra_context={"idea": idea, "best_score": best_score},
        )
        return ProblemSolutionResult(
            idea=idea,
            problem_md=problem_md,
            main_rs=best_code.rstrip() + "\n",
            last_eval=best_eval,
            verified=False,
        )

    raise RuntimeError(f"Failed to solve idea {idea!r} within {max_repair_attempts} attempts and no code was produced.")


def process_sample(
    snippet_code: str,
    output_dir: Optional[Path],
    produced_ideas: Optional[Set[str]],
    max_repair_attempts: int,
    max_problems: int,
) -> List[ProblemSolutionResult]:
    produced = produced_ideas if produced_ideas is not None else set()

    if output_dir is not None:
        manifest_path = output_dir / "manifest.jsonl"
        if manifest_path.exists():
            produced |= load_produced_ideas(output_dir)
        output_dir.mkdir(parents=True, exist_ok=True)
        (output_dir / "problems").mkdir(parents=True, exist_ok=True)
        (output_dir / "solutions").mkdir(parents=True, exist_ok=True)

    snippet_stem = output_dir.name if output_dir is not None else "snippet"
    eval_base = env("RUST_EVAL_BASE_URL", "http://127.0.0.1:3002")
    rustdocs_base = env("RUSTDOCS_BASE_URL", "http://127.0.0.1:3001")
    run_id = uuid.uuid4().hex[:8]
    recorder = StepRecorder(run_id=run_id, output_dir=output_dir)

    results: List[ProblemSolutionResult] = []
    with httpx.Client(timeout=120.0) as client:
        eval_server_warmup(eval_base, client)

        for _ in range(max(0, max_problems)):
            try:
                generated = generate_one_problem(
                    snippet_code=snippet_code,
                    previously_generated=results,
                    max_repair_attempts=max_repair_attempts,
                    eval_base=eval_base,
                    rustdocs_base=rustdocs_base,
                    client=client,
                    recorder=recorder,
                    run_id=run_id,
                )
            except Exception as exc:
                LOGGER.warning("process_sample generate_one_problem_failed run_id=%s error=%s", run_id, exc)
                continue

            if generated is None:
                LOGGER.info("process_sample ideation_opt_out run_id=%s generated=%s", run_id, len(results))
                break

            if generated.idea in produced:
                LOGGER.warning("process_sample duplicate_idea_skipped run_id=%s idea=%r", run_id, generated.idea)
                continue

            if not generated.verified:
                LOGGER.warning(
                    "process_sample best_effort_skipped run_id=%s idea=%r eval=%s",
                    run_id,
                    generated.idea,
                    preview_text(generated.last_eval, limit=300),
                )
                continue

            produced.add(generated.idea)
            results.append(generated)

            if output_dir is not None:
                item_id = str(uuid.uuid4())
                problem_path = output_dir / "problems" / f"{item_id}.md"
                solution_path = output_dir / "solutions" / f"{item_id}.rs"
                manifest_path = output_dir / "manifest.jsonl"

                problem_path.write_text(generated.problem_md, encoding="utf-8")
                solution_path.write_text(generated.main_rs, encoding="utf-8")
                with manifest_path.open("a", encoding="utf-8") as handle:
                    handle.write(
                        json.dumps(
                            {
                                "id": item_id,
                                "idea": generated.idea,
                                "source": snippet_stem,
                                "ok": True,
                            },
                            ensure_ascii=False,
                        )
                        + "\n"
                    )

            LOGGER.info(
                "process_sample problem_saved run_id=%s idea=%r problem_len=%s solution_len=%s",
                run_id,
                generated.idea,
                len(generated.problem_md),
                len(generated.main_rs),
            )

    return results


def _process_one_sample_file(
    sample_path: Path,
    output_root: Path,
    overwrite: bool,
    max_attempts: int,
    max_problems: int,
    produced_ideas: Optional[Set[str]] = None,
) -> int:
    sample_output_dir = output_root / sample_path.stem
    if sample_output_dir.exists() and not overwrite:
        manifest_path = sample_output_dir / "manifest.jsonl"
        has_existing = manifest_path.exists() or any((sample_output_dir / "solutions").glob("*.rs"))
        if has_existing:
            LOGGER.info("Skipping %s (output exists, use --overwrite to regenerate).", sample_path)
            return 0

    sample_code = sample_path.read_text(encoding="utf-8")
    results = process_sample(
        snippet_code=sample_code,
        output_dir=sample_output_dir,
        produced_ideas=produced_ideas,
        max_repair_attempts=max_attempts,
        max_problems=max_problems,
    )
    LOGGER.info("Completed sample=%s generated=%s problem+solution pairs", sample_path, len(results))
    return len(results)


if __name__ == "__main__":
    configure_logging()

    parser = argparse.ArgumentParser(
        description="Generate novel Win32 problems from snippets and compile-validated Rust solutions."
    )
    parser.add_argument(
        "--input",
        required=True,
        help="Path to a .rs file or directory of .rs files containing source snippets.",
    )
    parser.add_argument(
        "--output-dir",
        default="./problems_out",
        help="Root output directory for generated problems/solutions per snippet.",
    )
    parser.add_argument(
        "--max-problems",
        type=int,
        default=10,
        help="Maximum number of problem+solution pairs to generate per snippet.",
    )
    parser.add_argument(
        "--max-attempts",
        type=int,
        default=8,
        help="Max repair attempts per generated solution.",
    )
    parser.add_argument(
        "--overwrite",
        action="store_true",
        help="Re-generate even if output exists.",
    )
    parser.add_argument(
        "--workers",
        type=int,
        default=1,
        help="Maximum number of concurrent snippet workers.",
    )
    args = parser.parse_args()

    input_path = Path(args.input)
    output_root = Path(args.output_dir)
    sample_paths = _collect_input_samples(input_path)
    shared_produced_ideas = load_produced_ideas(output_root)
    LOGGER.info("pre_loaded_ideas total=%s", len(shared_produced_ideas))

    LOGGER.info(
        "generate_problems_agent start input=%s samples=%s output_dir=%s max_problems=%s max_attempts=%s overwrite=%s workers=%s cwd=%s",
        input_path,
        len(sample_paths),
        output_root,
        args.max_problems,
        args.max_attempts,
        args.overwrite,
        args.workers,
        os.getcwd(),
    )

    workers = max(1, int(args.workers))
    if workers == 1:
        for sample_path in sample_paths:
            _process_one_sample_file(
                sample_path=sample_path,
                output_root=output_root,
                overwrite=args.overwrite,
                max_attempts=args.max_attempts,
                max_problems=args.max_problems,
                produced_ideas=shared_produced_ideas,
            )
    else:
        with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as executor:
            futures: Dict[concurrent.futures.Future[int], Path] = {}
            for sample_path in sample_paths:
                future = executor.submit(
                    _process_one_sample_file,
                    sample_path=sample_path,
                    output_root=output_root,
                    overwrite=args.overwrite,
                    max_attempts=args.max_attempts,
                    max_problems=args.max_problems,
                    produced_ideas=None,
                )
                futures[future] = sample_path

            for future in concurrent.futures.as_completed(futures):
                sample_path = futures[future]
                try:
                    future.result()
                except Exception as exc:
                    LOGGER.exception("sample_failed path=%s error=%s", sample_path, exc)
