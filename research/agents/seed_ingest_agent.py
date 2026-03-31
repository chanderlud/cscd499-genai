import argparse
import json
import re
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, Optional

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

SYSTEM_PROMPT = """You are an expert Rust engineer specializing in Win32/Windows API programming using the `windows` crate.

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


@dataclass
class SeedIngestResult:
    problem_md: str
    solution_rs: str
    tests_rs: str
    last_eval: Dict[str, Any]
    verified: bool

def extract_seed_sections(seed_text: str) -> tuple[str, str, str]:
    def _extract_fenced_blocks(text: str, lang_pattern: str) -> list[str]:
        blocks: list[str] = []
        opening_pattern = re.compile(
            rf"^[ \t]{{0,3}}(`{{3,}})(?:{lang_pattern})[ \t]*$",
            re.MULTILINE | re.IGNORECASE,
        )

        for opening_match in opening_pattern.finditer(text):
            fence_len = len(opening_match.group(1))
            closing_pattern = re.compile(
                rf"^[ \t]{{0,3}}`{{{fence_len},}}[ \t]*$",
                re.MULTILINE,
            )
            closing_match = closing_pattern.search(text, opening_match.end())
            if closing_match:
                blocks.append(text[opening_match.end() : closing_match.start()])

        return blocks

    problem_open_match = re.search(
        r"^[ \t]{0,3}(`{3,})(markdown|md)[ \t]*$",
        seed_text,
        re.MULTILINE | re.IGNORECASE,
    )
    if not problem_open_match:
        raise ValueError("Missing markdown problem block in seed file.")

    fence = problem_open_match.group(1)
    problem_start = problem_open_match.end()
    remainder = seed_text[problem_start:]
    inner_fence_pattern = re.compile(r"^[ \t]{0,3}(`{3,})\w*[ \t]*$")
    outer_fence_pattern = re.compile(rf"^[ \t]{{0,3}}`{{{len(fence)},}}[ \t]*$")
    inner_open_len: Optional[int] = None
    problem_close_start: Optional[int] = None
    problem_close_end: Optional[int] = None
    offset = 0

    for line in remainder.splitlines(keepends=True):
        stripped_line = line.rstrip("\r\n")
        handled_inner_fence = False
        inner_fence_match = inner_fence_pattern.match(stripped_line)
        if inner_fence_match:
            inner_fence_len = len(inner_fence_match.group(1))
            if inner_open_len is None:
                if inner_fence_len < len(fence):
                    inner_open_len = inner_fence_len
                    handled_inner_fence = True
            elif inner_fence_len >= inner_open_len:
                inner_open_len = None
                handled_inner_fence = True

        if (
            not handled_inner_fence
            and inner_open_len is None
            and outer_fence_pattern.match(stripped_line)
        ):
            problem_close_start = offset
            problem_close_end = offset + len(line)
            break

        offset += len(line)

    if problem_close_start is None or problem_close_end is None:
        raise ValueError("Missing closing markdown problem fence in seed file.")

    raw_problem = remainder[:problem_close_start]
    problem_md = raw_problem.strip()
    after_markdown = remainder[problem_close_end:]

    rust_blocks = _extract_fenced_blocks(after_markdown, "rust|rs")
    if not rust_blocks:
        raise ValueError("Missing Rust code blocks in seed file.")

    solution_rs = next(
        (
            block.strip()
            for block in rust_blocks
            if "pub fn" in block and "#[cfg(test)]" not in block
        ),
        None,
    )
    if solution_rs is None:
        raise ValueError("Missing Rust solution block containing `pub fn` and no `#[cfg(test)]`.")

    tests_rs = next((block.strip() for block in rust_blocks if "#[cfg(test)]" in block), None)
    if tests_rs is None:
        raise ValueError("Missing Rust test block containing `#[cfg(test)]`.")

    return problem_md, solution_rs, tests_rs


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


def verify_and_repair(
    problem_md: str,
    solution_rs: str,
    tests_rs: str,
    max_attempts: int = 8,
    output_dir: Optional[Path] = None,
    problem_id: Optional[str] = None,
) -> SeedIngestResult:
    run_id = uuid.uuid4().hex[:8]
    eval_base = env("RUST_EVAL_BASE_URL", "http://127.0.0.1:3002")
    rustdocs_base = env("RUSTDOCS_BASE_URL", "http://127.0.0.1:3001")
    recorder = StepRecorder(run_id=run_id, output_dir=output_dir, problem_id=problem_id)

    LOGGER.info(
        "verify_and_repair start run_id=%s attempts=%s problem_len=%s solution_len=%s tests_len=%s",
        run_id,
        max_attempts,
        len(problem_md),
        len(solution_rs),
        len(tests_rs),
    )

    best_code = solution_rs
    best_eval: Dict[str, Any] = {}
    best_score = 10**9
    previous_code = ""
    same_streak = 0
    repair_context = ""

    with httpx.Client(timeout=120.0) as client:
        eval_server_warmup(eval_base, client)

        code = solution_rs
        for attempt in range(1, max_attempts + 1):
            if attempt > 1:
                user_prompt = REPAIR_PROMPT_TEMPLATE.format(context=repair_context)
                messages = [
                    {"role": "system", "content": SYSTEM_PROMPT},
                    {"role": "user", "content": user_prompt},
                ]

                response_text: Optional[str] = None
                for retry in range(2):
                    response_text = openrouter_generate_code(messages)
                    if response_text is not None:
                        break
                    if retry == 0:
                        retry_prompt = user_prompt + "\n\nPlease generate code. Output only a ```rust code block."
                        messages = [
                            {"role": "system", "content": SYSTEM_PROMPT},
                            {"role": "user", "content": retry_prompt},
                        ]

                if response_text is None:
                    LOGGER.warning("attempt=%s model_generation_failed", attempt)
                    continue

                extracted = extract_rust_code_block(response_text)
                if extracted is None:
                    recorder.record_step(
                        attempt=attempt,
                        step_type="no_code",
                        code="",
                        eval_result=None,
                        extra_context={"response_preview": preview_text(response_text, limit=500)},
                    )
                    repair_context = (
                        "## Build/Test Results\n"
                        "No Rust code block was generated in the previous attempt.\n\n"
                        "## Repair Instructions\n"
                        "- Output the complete src/main.rs in a single ```rust code fence.\n"
                    )
                    continue

                code = extracted
                recorder.record_step(
                    attempt=attempt,
                    step_type="generate",
                    code=code,
                    eval_result=None,
                    extra_context={"phase": "repair"},
                )
            else:
                recorder.record_step(
                    attempt=attempt,
                    step_type="generate",
                    code=code,
                    eval_result=None,
                    extra_context={"phase": "seed"},
                )

            symbols = extract_windows_api_symbols(code)
            rustdoc_info = ""
            try:
                rustdoc_info = batch_rustdoc_lookup(symbols, rustdocs_base, client)
            except Exception as exc:
                LOGGER.warning("attempt=%s rustdoc_lookup_unavailable error=%s", attempt, exc)
                rustdoc_info = ""

            eval_result: Dict[str, Any] = {}
            eval_error: Optional[Exception] = None
            for eval_try in range(2):
                try:
                    eval_result = eval_server_evaluate(
                        main_rs=code,
                        unit_tests_private=tests_rs,
                        fixed_deps=FIXED_DEPENDENCIES,
                        eval_base=eval_base,
                        client=client,
                        run_tests=True,
                    )
                    eval_error = None
                    break
                except (httpx.TimeoutException, httpx.HTTPStatusError) as exc:
                    eval_error = exc
                    LOGGER.warning("attempt=%s eval_retry=%s error=%s", attempt, eval_try + 1, exc)
                except Exception as exc:
                    eval_error = exc
                    LOGGER.warning("attempt=%s eval_failed error=%s", attempt, exc)
                    break

            if eval_error is not None and not eval_result:
                recorder.record_step(
                    attempt=attempt,
                    step_type="eval_error",
                    code=code,
                    eval_result=None,
                    extra_context={"error": str(eval_error)},
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
                extra_context={"symbols": symbols, "rustdoc_info": rustdoc_info},
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
                    LOGGER.warning("attempt=%s format_failed error=%s", attempt, exc)
                    formatted = None

                if formatted and formatted.strip() != code.strip():
                    recorder.record_step(
                        attempt=attempt,
                        step_type="format",
                        code=formatted,
                        eval_result=None,
                        extra_context={},
                    )
                    try:
                        formatted_eval = eval_server_evaluate(
                            main_rs=formatted,
                            unit_tests_private=tests_rs,
                            fixed_deps=FIXED_DEPENDENCIES,
                            eval_base=eval_base,
                            client=client,
                            run_tests=True,
                        )
                    except Exception as exc:
                        LOGGER.warning("attempt=%s formatted_recheck_failed error=%s", attempt, exc)
                        formatted_eval = None

                    if isinstance(formatted_eval, dict) and formatted_eval.get("ok") is True:
                        recorder.record_final(formatted, formatted_eval)
                        return SeedIngestResult(
                            problem_md=problem_md,
                            solution_rs=formatted.rstrip() + "\n",
                            tests_rs=tests_rs.rstrip() + "\n",
                            last_eval=formatted_eval,
                            verified=True,
                        )

                    recorder.record_final(code, eval_result)
                    return SeedIngestResult(
                        problem_md=problem_md,
                        solution_rs=code.rstrip() + "\n",
                        tests_rs=tests_rs.rstrip() + "\n",
                        last_eval=eval_result,
                        verified=True,
                    )

                recorder.record_final(code, eval_result)
                return SeedIngestResult(
                    problem_md=problem_md,
                    solution_rs=code.rstrip() + "\n",
                    tests_rs=tests_rs.rstrip() + "\n",
                    last_eval=eval_result,
                    verified=True,
                )

            diagnostic_symbols = extract_symbols_from_diagnostics(eval_result)
            targeted_info = ""
            if diagnostic_symbols:
                try:
                    targeted_info = batch_rustdoc_lookup(diagnostic_symbols, rustdocs_base, client)
                except Exception as exc:
                    LOGGER.warning("attempt=%s targeted_lookup_failed error=%s", attempt, exc)

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

    LOGGER.warning(
        "verify_and_repair exhausted_attempts returning_best run_id=%s best_score=%s",
        run_id,
        best_score,
    )
    recorder.record_step(
        attempt=max_attempts + 1,
        step_type="best_effort",
        code=best_code,
        eval_result=best_eval,
        extra_context={"best_score": best_score},
    )
    return SeedIngestResult(
        problem_md=problem_md,
        solution_rs=best_code.rstrip() + "\n",
        tests_rs=tests_rs.rstrip() + "\n",
        last_eval=best_eval,
        verified=False,
    )


def save_result(
    result: SeedIngestResult,
    output_dir: Path,
    problem_id: str,
) -> None:
    problems_dir = output_dir / "problems"
    solutions_dir = output_dir / "solutions"
    tests_dir = output_dir / "tests"

    problems_dir.mkdir(parents=True, exist_ok=True)
    solutions_dir.mkdir(parents=True, exist_ok=True)
    tests_dir.mkdir(parents=True, exist_ok=True)

    (problems_dir / f"{problem_id}.md").write_text(result.problem_md.rstrip() + "\n", encoding="utf-8")
    (solutions_dir / f"{problem_id}.rs").write_text(result.solution_rs, encoding="utf-8")
    (tests_dir / f"{problem_id}.rs").write_text(result.tests_rs, encoding="utf-8")

    manifest_record = {
        "id": problem_id,
        "ok": result.verified,
        "verified": result.verified,
        "eval_ok": result.last_eval.get("ok") is True,
    }
    with (output_dir / "manifest.jsonl").open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(manifest_record, ensure_ascii=False) + "\n")


def ingest_seed(
    seed_path: Path,
    output_dir: Path,
    max_attempts: int,
    overwrite: bool,
) -> None:
    problem_id = str(uuid.uuid4())
    problem_path = output_dir / "problems" / f"{problem_id}.md"
    if problem_path.exists() and not overwrite:
        LOGGER.info("ingest_seed skip id=%s reason=exists", problem_id)
        return

    seed_text = seed_path.read_text(encoding="utf-8")
    problem_md, solution_rs, tests_rs = extract_seed_sections(seed_text)
    steps_dir = output_dir / "steps" / problem_id
    steps_dir.mkdir(parents=True, exist_ok=True)
    (steps_dir / "seed_problem.md").write_text(problem_md, encoding="utf-8")
    (steps_dir / "seed_solution.rs").write_text(solution_rs, encoding="utf-8")
    (steps_dir / "seed_tests.rs").write_text(tests_rs, encoding="utf-8")
    LOGGER.info(
        "ingest_seed extracted id=%s problem_len=%s solution_len=%s tests_len=%s",
        problem_id,
        len(problem_md),
        len(solution_rs),
        len(tests_rs),
    )
    result = verify_and_repair(
        problem_md=problem_md,
        solution_rs=solution_rs,
        tests_rs=tests_rs,
        max_attempts=max_attempts,
        output_dir=output_dir,
        problem_id=problem_id,
    )
    if not result.verified:
        LOGGER.warning("ingest_seed verification_failed id=%s eval_ok=%s", problem_id, result.last_eval.get("ok"))
        return

    save_result(result=result, output_dir=output_dir, problem_id=problem_id)
    LOGGER.info("ingest_seed saved id=%s output_dir=%s", problem_id, output_dir)


def ingest_seed_directory(
    seed_dir: Path,
    output_dir: Path,
    max_attempts: int,
    overwrite: bool,
) -> None:
    seed_files = sorted(seed_dir.glob("*.md"))
    LOGGER.info("ingest_seed_directory discovered=%s seed_dir=%s", len(seed_files), seed_dir)
    for seed_path in seed_files:
        try:
            ingest_seed(
                seed_path=seed_path,
                output_dir=output_dir,
                max_attempts=max_attempts,
                overwrite=overwrite,
            )
        except Exception as exc:
            LOGGER.exception("ingest_seed_directory failed seed=%s error=%s", seed_path, exc)


if __name__ == "__main__":
    configure_logging()
    parser = argparse.ArgumentParser(description="Seed ingest agent")
    seed_group = parser.add_mutually_exclusive_group(required=True)
    seed_group.add_argument("--seed", default=None, help="Path to the seed .md file")
    seed_group.add_argument("--seed-dir", default=None, help="Directory of seed .md files to ingest")
    parser.add_argument("--output-dir", default="./seed_out", help="Root output directory")
    parser.add_argument("--max-attempts", type=int, default=8, help="Max repair loop iterations")
    parser.add_argument(
        "--overwrite",
        action="store_true",
        help="Re-process even if output exists.",
    )
    args = parser.parse_args()

    if args.seed:
        ingest_seed(
            seed_path=Path(args.seed),
            output_dir=Path(args.output_dir),
            max_attempts=args.max_attempts,
            overwrite=args.overwrite,
        )
    elif args.seed_dir:
        ingest_seed_directory(
            seed_dir=Path(args.seed_dir),
            output_dir=Path(args.output_dir),
            max_attempts=args.max_attempts,
            overwrite=args.overwrite,
        )
