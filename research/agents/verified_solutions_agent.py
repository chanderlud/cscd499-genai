import argparse
import os
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
    load_resume_state,
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
class SolveResult:
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


def solve_problem(
    problem_text: str,
    unit_tests_private: str,
    max_attempts: int = 8,
    output_dir: Optional[Path] = None,
    problem_id: Optional[str] = None,
    resume: bool = False,
) -> SolveResult:
    run_id = uuid.uuid4().hex[:8]
    eval_base = env("RUST_EVAL_BASE_URL", "http://127.0.0.1:3002")
    rustdocs_base = env("RUSTDOCS_BASE_URL", "http://127.0.0.1:3001")
    recorder = StepRecorder(run_id=run_id, output_dir=output_dir, problem_id=problem_id)

    LOGGER.info(
        "solve_problem start run_id=%s attempts=%s problem_len=%s tests_len=%s",
        run_id,
        max_attempts,
        len(problem_text),
        len(unit_tests_private),
    )

    best_code = ""
    best_eval: Dict[str, Any] = {}
    best_score = 10**9
    previous_code = ""
    same_streak = 0
    repair_context = ""
    start_attempt = 1

    if resume and problem_id:
        base = output_dir or Path("out")
        steps_dir = base / "steps" / problem_id
        resume_state = load_resume_state(steps_dir)
        if resume_state:
            start_attempt = int(resume_state["start_attempt"])
            resumed_code = resume_state["last_code"]
            resumed_eval = resume_state["last_eval"]
            best_code = resumed_code
            previous_code = resumed_code
            if isinstance(resumed_eval, dict):
                best_eval = resumed_eval
                best_score = _error_score(resumed_eval)
                repair_context = build_repair_context(
                    eval_result=resumed_eval,
                    main_rs=resumed_code,
                    rustdoc_info="",
                    problem_text=problem_text,
                )
            LOGGER.info(
                "solve_problem resume_detected run_id=%s problem_id=%s start_attempt=%s",
                run_id,
                problem_id,
                start_attempt,
            )

    with httpx.Client(timeout=120.0) as client:
        eval_server_warmup(eval_base, client)

        for attempt in range(start_attempt, max_attempts + 1):
            user_prompt = problem_text if attempt == 1 else REPAIR_PROMPT_TEMPLATE.format(context=repair_context)
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

            code = extract_rust_code_block(response_text)
            if code is None:
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

            recorder.record_step(
                attempt=attempt,
                step_type="generate",
                code=code,
                eval_result=None,
                extra_context={"phase": "initial" if attempt == 1 else "repair"},
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
                        unit_tests_private=unit_tests_private,
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
                            unit_tests_private=unit_tests_private,
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
                        return SolveResult(main_rs=formatted.rstrip() + "\n", last_eval=formatted_eval)

                    recorder.record_final(code, eval_result)
                    return SolveResult(main_rs=code.rstrip() + "\n", last_eval=eval_result)

                recorder.record_final(code, eval_result)
                return SolveResult(main_rs=code.rstrip() + "\n", last_eval=eval_result)

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
                problem_text=problem_text,
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
            "solve_problem exhausted_attempts returning_best run_id=%s best_score=%s",
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
        return SolveResult(main_rs=best_code.rstrip() + "\n", last_eval=best_eval, verified=False)

    raise RuntimeError(f"Failed to solve within {max_attempts} attempts and no valid code was produced.")


def process_input_folder(input_dir: Path, max_attempts: int, overwrite: bool, resume: bool = False) -> None:
    problems_dir = input_dir / "problems"
    tests_dir = input_dir / "tests"
    solutions_dir = input_dir / "solutions"
    output_dir = Path("out")

    if not problems_dir.is_dir():
        raise ValueError(f"Problems directory not found: {problems_dir}")
    if not tests_dir.is_dir():
        raise ValueError(f"Tests directory not found: {tests_dir}")

    solutions_dir.mkdir(parents=True, exist_ok=True)

    problem_files = sorted(problems_dir.glob("*.md"))
    LOGGER.info(
        "process_input_folder start input=%s count=%s max_attempts=%s overwrite=%s",
        input_dir,
        len(problem_files),
        max_attempts,
        overwrite,
    )

    for md_path in problem_files:
        problem_id = md_path.stem
        test_path = tests_dir / f"{problem_id}.rs"
        solution_out = solutions_dir / f"{problem_id}.rs"

        if solution_out.exists() and not overwrite:
            LOGGER.info("process_input_folder skip id=%s reason=exists", problem_id)
            continue

        if not test_path.exists():
            LOGGER.warning("process_input_folder skip id=%s reason=missing_test", problem_id)
            continue

        problem_text = md_path.read_text(encoding="utf-8")
        unit_tests_text = test_path.read_text(encoding="utf-8")

        try:
            result = solve_problem(
                problem_text=problem_text,
                unit_tests_private=unit_tests_text,
                max_attempts=max_attempts,
                output_dir=output_dir,
                problem_id=problem_id,
                resume=resume,
            )
            if result.verified:
                solution_out.write_text(result.main_rs, encoding="utf-8")
                LOGGER.info(
                    "process_input_folder ok id=%s eval_ok=%s",
                    problem_id,
                    result.last_eval.get("ok") is True,
                )
            else:
                LOGGER.warning(
                    "process_input_folder best_effort_skipped id=%s best_score=%s steps_dir=out/steps/%s",
                    problem_id,
                    result.last_eval,
                    problem_id,
                )
        except Exception as exc:
            LOGGER.exception("process_input_folder failed id=%s error=%s", problem_id, exc)
            continue


if __name__ == "__main__":
    configure_logging()
    parser = argparse.ArgumentParser(description="Batch verified solutions agent")
    parser.add_argument(
        "--input-dir",
        required=True,
        help="Directory containing problems/ and tests/ subdirectories. Solutions are written to solutions/ under this directory.",
    )
    parser.add_argument("--max-attempts", type=int, default=8)
    parser.add_argument(
        "--overwrite",
        action="store_true",
        help="Re-solve even if a solution already exists.",
    )
    parser.add_argument(
        "--resume",
        action="store_true",
        help="Resume from last saved step for each problem.",
    )
    args = parser.parse_args()

    process_input_folder(
        input_dir=Path(args.input_dir),
        max_attempts=args.max_attempts,
        overwrite=args.overwrite,
        resume=args.resume,
    )
