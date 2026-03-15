import argparse
import json
import os
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, List, Optional, Set

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

Your task is to extract one standalone example from existing sample code.

Output requirements:
- Output exactly one complete `src/main.rs` inside a single ```rust code fence.
- No explanation text outside the fence.
- Do not include tests.
- Use only stable Rust.

Code constraints:
- Use the `windows` crate for Win32 APIs and prefer W-suffix APIs where applicable.
- Minimize `unsafe` scope to the smallest possible block.
- Use proper error propagation for HRESULT/WIN32_ERROR patterns.
- Keep behavior focused on the sample's existing functionality.
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
- After a failing non-`Result` Win32 call, call `windows::core::Error::from_thread()` - no argument - to capture `GetLastError()` as a `windows::core::Error`.
- To convert a raw `u32` error code to an `HRESULT`, call `HRESULT::from_win32(code)` - one `u32` argument. Never call `Error::from_win32(code)` with an argument; that method does not exist.
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

// Inline example - converting a known constant:
fn example() -> HRESULT {
    HRESULT::from_win32(ERROR_ACCESS_DENIED.0)
}
```

Key rules:
- `WIN32_ERROR` has a `.0` field (the raw `u32`); pass it to `HRESULT::from_win32()` when you need the raw value.
- Prefer `err.to_hresult()` over `.0` when you already hold a `WIN32_ERROR` - it is more idiomatic.
- Never hard-code `HRESULT(0x80070005_u32 as i32)` or similar literals.

### DO NOT CONFUSE these APIs

// WRONG - Error has no from_win32(code) method:
//   windows::core::Error::from_win32(code)
//
// CORRECT - use HRESULT::from_win32 for a code, Error::from_thread() for GetLastError:
//   HRESULT::from_win32(code)          // takes a u32 argument
//   windows::core::Error::from_hresult(hresult) // takes a HRESULT argument
//   windows::core::Error::from_thread() // zero arguments - reads GetLastError()

### `windows::core::Error` - reads `GetLastError()` automatically (zero arguments)

Use this after a failing non-`Result` Win32 call:
- `windows::core::Error::from_win32()` takes no arguments.
- It captures the current thread's `GetLastError()` value as a `windows::core::Error`.
"""

EXTRACT_ONE_PROMPT_TEMPLATE = """Extract exactly one standalone Windows API usage example from the sample code below.

Rules:
- Extract what already exists in the sample. Do not invent new functionality.
- The output must be one complete `src/main.rs` file in a single ```rust code fence.
- Include a short imperative title on a line that starts with `TITLE:`.
- Keep the example focused and distinct from already extracted examples.

Previously extracted examples:
{previously_extracted}

If there are no more unique, distinct Windows API usage patterns left to extract that differ meaningfully from the ones already listed, respond with exactly the line `NO_MORE_EXAMPLES` and nothing else.

## Full Original Sample Code
```rust
{sample_code}
```
"""

REPAIR_PROMPT_TEMPLATE = """Your previous code attempt failed to compile. Fix ONLY the reported errors.
Use the Rustdoc symbol resolution below to correct any import paths.
Do NOT rewrite from scratch - make targeted fixes.

{context}

Windows repair reminders:
- To wrap a raw `u32` code as an `HRESULT`, use `HRESULT::from_win32(code)` (one argument) or `err.to_hresult()` for a `WIN32_ERROR` value.
- To capture `GetLastError()` as a `windows::core::Error`, use `windows::core::Error::from_thread()` with no argument. `Error::from_win32` does not accept a `u32` argument - that overload does not exist.
- Prefer W-suffix Win32 APIs and minimize `unsafe` scope.

Output the complete fixed src/main.rs in a single ```rust code fence.
"""


@dataclass
class SnippetResult:
    idea: str
    main_rs: str
    last_eval: Optional[Dict[str, Any]]


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


def _format_previous_examples(previously_extracted: List[SnippetResult]) -> str:
    if not previously_extracted:
        return "(none yet)"
    lines: List[str] = []
    for idx, item in enumerate(previously_extracted, start=1):
        symbols = extract_windows_api_symbols(item.main_rs)
        brief = ", ".join(symbols[:5]) if symbols else "no symbol summary available"
        lines.append(f"{idx}. {item.idea} - {brief}")
    return "\n".join(lines)


def _extract_title(response_text: str) -> str:
    for raw_line in response_text.splitlines():
        line = raw_line.strip()
        if line.startswith("TITLE:"):
            title = line[len("TITLE:") :].strip()
            if title:
                return title
    return "Extract standalone Windows API example"


def extract_one_snippet(
    sample_code: str,
    previously_extracted: List[SnippetResult],
    max_repair_attempts: int,
    eval_base: str,
    rustdocs_base: str,
    client: "httpx.Client",
    recorder: StepRecorder,
    run_id: str,
) -> Optional[SnippetResult]:
    previous_list = _format_previous_examples(previously_extracted)
    extract_prompt = EXTRACT_ONE_PROMPT_TEMPLATE.format(
        sample_code=sample_code,
        previously_extracted=previous_list,
    )

    messages = [
        {"role": "system", "content": SYSTEM_PROMPT},
        {"role": "user", "content": extract_prompt},
    ]

    response_text: Optional[str] = None
    for retry in range(2):
        response_text = openrouter_generate_code(messages)
        if response_text is not None:
            break
        if retry == 0:
            retry_prompt = extract_prompt + "\n\nPlease output `TITLE:` and a single ```rust code block."
            messages = [
                {"role": "system", "content": SYSTEM_PROMPT},
                {"role": "user", "content": retry_prompt},
            ]
    if response_text is None:
        LOGGER.warning("extract_one_snippet generation_failed run_id=%s", run_id)
        return None

    stripped_response = response_text.strip()
    if "NO_MORE_EXAMPLES" in stripped_response:
        recorder.record_step(
            attempt=1,
            step_type="extract_opt_out",
            code="",
            eval_result=None,
            extra_context={"response": preview_text(stripped_response, limit=300)},
        )
        return None

    title = _extract_title(response_text)
    code = extract_rust_code_block(response_text)
    recorder.record_step(
        attempt=1,
        step_type="extract_generate",
        code=code or "",
        eval_result=None,
        extra_context={"title": title, "response_preview": preview_text(response_text, limit=400)},
    )

    repair_context = ""
    previous_code = ""
    same_streak = 0
    last_eval: Optional[Dict[str, Any]] = None

    for attempt in range(1, max_repair_attempts + 1):
        if code is None:
            recorder.record_step(
                attempt=attempt,
                step_type="no_code",
                code="",
                eval_result=None,
                extra_context={"title": title},
            )
            repair_context = (
                "## Build/Test Results\n"
                "No Rust code block was generated in the previous attempt.\n\n"
                "## Repair Instructions\n"
                "- Output the complete src/main.rs in a single ```rust code fence.\n"
            )
        else:
            symbols = extract_windows_api_symbols(code)
            rustdoc_info = ""
            try:
                rustdoc_info = batch_rustdoc_lookup(symbols, rustdocs_base, client)
            except Exception as exc:
                LOGGER.warning("extract_one_snippet rustdoc_lookup_failed run_id=%s error=%s", run_id, exc)

            try:
                eval_result = eval_server_evaluate(
                    main_rs=code,
                    unit_tests_private="",
                    fixed_deps=FIXED_DEPENDENCIES,
                    eval_base=eval_base,
                    client=client,
                    run_tests=False,
                )
            except Exception as exc:
                recorder.record_step(
                    attempt=attempt,
                    step_type="eval_error",
                    code=code,
                    eval_result=None,
                    extra_context={"error": str(exc), "title": title},
                )
                repair_context = (
                    "## Build/Test Results\n"
                    f"Evaluator request failed: {exc}\n\n"
                    "## Repair Instructions\n"
                    "- Keep the same approach and output valid Rust in a single fence.\n"
                )
                eval_result = {}

            last_eval = eval_result
            recorder.record_step(
                attempt=attempt,
                step_type="eval",
                code=code,
                eval_result=eval_result,
                extra_context={"title": title, "symbols": symbols},
            )

            if eval_result.get("ok") is True:
                formatted = None
                try:
                    formatted = eval_server_format(code, eval_base, client)
                except Exception as exc:
                    LOGGER.warning("extract_one_snippet format_failed run_id=%s error=%s", run_id, exc)
                    formatted = None

                if formatted and formatted.strip() != code.strip():
                    recorder.record_step(
                        attempt=attempt,
                        step_type="format",
                        code=formatted,
                        eval_result=None,
                        extra_context={"title": title},
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
                            "extract_one_snippet formatted_recheck_failed run_id=%s error=%s",
                            run_id,
                            exc,
                        )
                        formatted_eval = None
                    if isinstance(formatted_eval, dict) and formatted_eval.get("ok") is True:
                        return SnippetResult(idea=title, main_rs=formatted.rstrip() + "\n", last_eval=formatted_eval)

                return SnippetResult(idea=title, main_rs=code.rstrip() + "\n", last_eval=eval_result)

            diagnostic_symbols = extract_symbols_from_diagnostics(eval_result)
            targeted_info = ""
            if diagnostic_symbols:
                try:
                    targeted_info = batch_rustdoc_lookup(diagnostic_symbols, rustdocs_base, client)
                except Exception as exc:
                    LOGGER.warning("extract_one_snippet targeted_lookup_failed run_id=%s error=%s", run_id, exc)

            combined_info_parts = [part for part in [rustdoc_info, targeted_info] if part.strip()]
            combined_info = "\n\n".join(combined_info_parts)
            repair_context = build_repair_context(
                eval_result=eval_result,
                main_rs=code,
                rustdoc_info=combined_info,
                problem_text=title,
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

        repair_user_msg = REPAIR_PROMPT_TEMPLATE.format(context=repair_context)
        repair_messages = [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": repair_user_msg},
        ]
        repair_response = openrouter_generate_code(repair_messages)
        if repair_response is None:
            LOGGER.warning(
                "extract_one_snippet repair_generation_failed run_id=%s attempt=%s",
                run_id,
                attempt,
            )
            continue
        repaired_code = extract_rust_code_block(repair_response)
        recorder.record_step(
            attempt=attempt,
            step_type="repair_generate",
            code=repaired_code or "",
            eval_result=None,
            extra_context={"title": title, "response_preview": preview_text(repair_response, limit=400)},
        )
        code = repaired_code

    LOGGER.warning(
        "extract_one_snippet exhausted_repair_attempts run_id=%s title=%r last_eval=%r",
        run_id,
        title,
        preview_text(last_eval, limit=400),
    )
    return None


def load_produced_ideas(output_root: Path, include_failed: bool = False) -> Set[str]:
    """Load all persisted idea strings from every manifest.jsonl under output_root."""
    ideas: Set[str] = set()
    manifests = list(output_root.rglob("manifest.jsonl"))
    for manifest_path in manifests:
        count_before = len(ideas)
        try:
            with manifest_path.open("r", encoding="utf-8") as f:
                for line in f:
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
        except OSError as e:
            LOGGER.warning("load_produced_ideas skip file=%s error=%s", manifest_path, e)
    LOGGER.info("load_produced_ideas total manifests=%s total_ideas=%s", len(manifests), len(ideas))
    return ideas


def process_sample(
    sample_code: str,
    output_dir: Optional[Path] = None,
    produced_ideas: Optional[Set[str]] = None,
    max_repair_attempts: int = 8,
    max_snippets: int = 20,
) -> List[SnippetResult]:
    produced = produced_ideas if produced_ideas is not None else set()
    if output_dir is not None:
        manifest_path = output_dir / "manifest.jsonl"
        if manifest_path.exists():
            produced |= load_produced_ideas(output_dir)

    if output_dir:
        output_dir.mkdir(parents=True, exist_ok=True)

    eval_base = env("RUST_EVAL_BASE_URL", "http://127.0.0.1:3002")
    rustdocs_base = env("RUSTDOCS_BASE_URL", "http://127.0.0.1:3001")
    run_id = uuid.uuid4().hex[:8]
    recorder = StepRecorder(run_id=run_id, output_dir=output_dir)

    results: List[SnippetResult] = []
    httpx_client_cls = __import__("httpx").Client
    with httpx_client_cls(timeout=120.0) as client:
        eval_server_warmup(eval_base, client)

        for _ in range(max(0, max_snippets)):
            try:
                extracted = extract_one_snippet(
                    sample_code=sample_code,
                    previously_extracted=results,
                    max_repair_attempts=max_repair_attempts,
                    eval_base=eval_base,
                    rustdocs_base=rustdocs_base,
                    client=client,
                    recorder=recorder,
                    run_id=run_id,
                )
            except Exception as exc:
                LOGGER.warning("process_sample extract_one_snippet_failed run_id=%s error=%s", run_id, exc)
                continue

            if extracted is None:
                LOGGER.info("process_sample model_opt_out run_id=%s extracted=%s", run_id, len(results))
                break

            if extracted.idea in produced:
                LOGGER.warning("process_sample duplicate_title_skipped run_id=%s title=%r", run_id, extracted.idea)
                continue

            produced.add(extracted.idea)
            results.append(extracted)

            if output_dir:
                snippet_id = uuid.uuid4().hex[:8]
                out_path = output_dir / f"{snippet_id}.rs"
                out_path.write_text(extracted.main_rs, encoding="utf-8")

                manifest_path = output_dir / "manifest.jsonl"
                with manifest_path.open("a", encoding="utf-8") as f:
                    f.write(json.dumps({"id": snippet_id, "idea": extracted.idea, "ok": True}) + "\n")

            LOGGER.info(
                "process_sample snippet_saved run_id=%s title=%r snippet_len=%s",
                run_id,
                extracted.idea,
                len(extracted.main_rs),
            )

    return results


def _collect_input_samples(input_path: Path) -> List[Path]:
    if input_path.is_file() and input_path.suffix.lower() == ".rs":
        return [input_path]
    if input_path.is_dir():
        return sorted(path for path in input_path.rglob("*.rs") if path.is_file())
    raise ValueError(f"Input must be a .rs file or directory of .rs files: {input_path}")


if __name__ == "__main__":
    configure_logging()

    parser = argparse.ArgumentParser(
        description="Extract standalone Windows API usage examples from raw Rust sample files."
    )
    parser.add_argument(
        "--input",
        required=True,
        help="Path to a .rs file or directory of .rs files containing sample code.",
    )
    parser.add_argument(
        "--output-dir",
        default="./snippets_out",
        help="Directory to write validated snippets.",
    )
    parser.add_argument(
        "--max-usages",
        type=int,
        default=20,
        help="Maximum number of snippets to extract per input sample.",
    )
    parser.add_argument(
        "--max-attempts",
        type=int,
        default=8,
        help="Max repair attempts per snippet extraction.",
    )
    parser.add_argument(
        "--overwrite",
        action="store_true",
        help="Re-generate even if output exists.",
    )
    args = parser.parse_args()

    input_path = Path(args.input)
    output_root = Path(args.output_dir)
    sample_paths = _collect_input_samples(input_path)
    shared_produced_ideas = load_produced_ideas(output_root)
    LOGGER.info("pre_loaded_ideas total=%s", len(shared_produced_ideas))

    LOGGER.info(
        "extract_snippets_agent start input=%s samples=%s output_dir=%s max_usages=%s max_attempts=%s overwrite=%s cwd=%s",
        input_path,
        len(sample_paths),
        output_root,
        args.max_usages,
        args.max_attempts,
        args.overwrite,
        os.getcwd(),
    )

    for sample_path in sample_paths:
        LOGGER.info("Extracting snippets from %s", sample_path)
        sample_output_dir = output_root / sample_path.stem

        if sample_output_dir.exists() and not args.overwrite:
            manifest_path = sample_output_dir / "manifest.jsonl"
            has_existing = manifest_path.exists() or any(sample_output_dir.glob("*.rs"))
            if has_existing:
                LOGGER.info("Skipping %s (output exists, use --overwrite to regenerate).", sample_path)
                continue

        sample_code = sample_path.read_text(encoding="utf-8")
        results = process_sample(
            sample_code=sample_code,
            output_dir=sample_output_dir,
            produced_ideas=shared_produced_ideas,
            max_repair_attempts=args.max_attempts,
            max_snippets=args.max_usages,
        )
        LOGGER.info("Completed sample=%s generated=%s snippets", sample_path, len(results))
