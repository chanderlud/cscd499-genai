import argparse
import json
import os
import re
import time
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, Optional, Tuple

import httpx

from helpers import (
    LOGGER,
    RUST_FENCE_RE,
    batch_rustdoc_lookup,
    build_repair_context,
    build_repair_message,
    code_help_tool,
    configure_logging,
    ensure_empty_main,
    env,
    eval_server_evaluate,
    eval_server_format,
    eval_server_warmup,
    extract_rust_code_block,
    extract_symbols_from_diagnostics,
    extract_windows_api_symbols,
    normalize_rust_text,
    openrouter_generate_code,
    preview_text,
    truncate_feedback,
)


FIXED_DEPENDENCIES = (
    Path(__file__).resolve().parent.parent / "rust_dependencies.md"
).read_text(encoding="utf-8")

MARKDOWN_FENCE_RE = re.compile(
    r"```(?:markdown|md)\s*\n(?P<content>[\s\S]*)\n\s*```",
    re.IGNORECASE,
)
UPDATED_TESTS_SECTION_RE = re.compile(r"##\s*UPDATED\s+TESTS", re.IGNORECASE)

REPAIR_PROMPT_TEMPLATE = """Your previous test module failed to compile with the generated stub.
Fix ONLY the reported errors and keep the test intent aligned with the problem statement.
Use the Rustdoc symbol resolution below to correct import paths.
Do NOT rewrite from scratch - make targeted fixes.

{context}

Output the complete fixed test module in a single ```rust code fence.
"""


@dataclass
class AlignmentResult:
    problem_md: str
    test_rs: str
    last_eval: Dict[str, Any]
    skipped: bool


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


def review_problem_and_tests(
    problem_text: str,
    test_rs: str,
    run_id: str,
) -> Tuple[Optional[str], Optional[str], Optional[str]]:
    system_prompt = (
        "You are an expert Rust/Windows problem-quality reviewer.\n"
        "Review the problem and unit tests. Ensure they are sound, unambiguous, and well aligned.\n"
        "Output updated unit tests and problem markdown if needed.\n"
        "Maintain the original formatting of specs/constraints/signature/example.\n"
        "Do not include any ```rust fenced blocks inside ## PROBLEM MARKDOWN; use ```text for code-like examples there."
    )
    user_prompt = f"""Review and align the following problem and tests.

Return output in this exact structure:
## PROBLEM MARKDOWN
```markdown
<full updated problem markdown>
```

## UPDATED TESTS
```rust
<full updated rust test module>
```

If this pair should be skipped, respond with:
SKIP: <one-line reason>

## Input Problem
```markdown
{problem_text}
```

## Input Tests
```rust
{test_rs}
```
"""
    messages = [
        {"role": "system", "content": system_prompt},
        {"role": "user", "content": user_prompt},
    ]
    response = openrouter_generate_code(messages)
    if response is None or not response.strip():
        return None, None, "empty OpenRouter response"

    text = response.strip()
    if text.startswith("SKIP:"):
        reason = text[len("SKIP:") :].strip() or "model requested skip"
        return None, None, reason

    parts = UPDATED_TESTS_SECTION_RE.split(text, maxsplit=1)
    problem_section = parts[0]
    tests_section = parts[1] if len(parts) == 2 else None

    md_match = MARKDOWN_FENCE_RE.search(problem_section)
    updated_problem = md_match.group("content").strip() if md_match else None
    updated_tests = extract_rust_code_block(tests_section) if tests_section is not None else None

    if not updated_problem or not updated_tests:
        LOGGER.warning(
            "review_problem_and_tests parse_failed run_id=%s preview=%s",
            run_id,
            preview_text(text, limit=500),
        )
        return None, None, "could not parse response"

    return updated_problem.rstrip() + "\n", updated_tests.rstrip() + "\n", None


def generate_stub_solution(problem_text: str) -> str:
    run_id = uuid.uuid4().hex[:8]
    prompt = f"""You are an expert Rust engineer.

Given the problem statement below, generate a minimal Rust stub implementation suitable only for compiling/running unit tests.

Rules:
- Implement the function signature(s) described by the problem.
- The body must use unimplemented!().
- Do not include any imports not required for the function signature.
- Always include use windows::core::{{Result, Error}} even if unused.
- Include fn main() {{}}.
- Output only one Rust fenced code block.

## Problem
{problem_text}
"""
    answer = code_help_tool(prompt, run_id)
    if not answer or not answer.strip():
        return "fn main() {}\n"
    match = RUST_FENCE_RE.search(answer)
    stub = match.group(1).strip() if match else answer.strip()
    return ensure_empty_main(stub.rstrip() + "\n")


def verify_tests_compile(
    test_rs: str,
    stub_solution: str,
    fixed_deps: str,
    eval_base: str,
    client: httpx.Client,
) -> Dict[str, Any]:
    normalized_test_rs = normalize_rust_text(test_rs, field_name="test_rs")
    full_main = ensure_empty_main(stub_solution).rstrip() + "\n\n" + normalized_test_rs.strip() + "\n"
    return eval_server_evaluate(
        main_rs=full_main,
        unit_tests_private="",
        fixed_deps=fixed_deps,
        eval_base=eval_base,
        client=client,
        run_tests=False,
    )


def repair_tests_loop(
    problem_text: str,
    test_rs: str,
    stub_solution: str,
    max_repair_attempts: int,
    eval_base: str,
    rustdocs_base: str,
    client: httpx.Client,
    fixed_deps: str,
) -> Tuple[str, Dict[str, Any]]:
    current_test_rs = test_rs.rstrip() + "\n"
    best_test_rs = current_test_rs
    best_eval: Dict[str, Any] = {}
    best_score = 10**9
    previous_code = ""
    same_streak = 0

    for attempt in range(1, max_repair_attempts + 1):
        eval_result = verify_tests_compile(
            test_rs=current_test_rs,
            stub_solution=stub_solution,
            fixed_deps=fixed_deps,
            eval_base=eval_base,
            client=client,
        )
        eval_result["_attempts"] = attempt
        score = _error_score(eval_result)
        if score < best_score:
            best_score = score
            best_test_rs = current_test_rs
            best_eval = eval_result

        if eval_result.get("ok") is True:
            try:
                formatted = eval_server_format(current_test_rs, eval_base, client)
            except Exception as exc:
                LOGGER.warning("repair_tests_loop format_failed attempt=%s error=%s", attempt, exc)
                formatted = None
            if formatted and formatted.strip():
                return formatted.rstrip() + "\n", eval_result
            return current_test_rs.rstrip() + "\n", eval_result

        diagnostic_symbols = extract_symbols_from_diagnostics(eval_result)
        targeted_info = ""
        if diagnostic_symbols:
            try:
                targeted_info = batch_rustdoc_lookup(diagnostic_symbols, rustdocs_base, client)
            except Exception as exc:
                LOGGER.warning(
                    "repair_tests_loop targeted_lookup_failed attempt=%s error=%s",
                    attempt,
                    exc,
                )

        code_symbols = extract_windows_api_symbols(current_test_rs)
        broad_info = ""
        if code_symbols:
            try:
                broad_info = batch_rustdoc_lookup(code_symbols, rustdocs_base, client)
            except Exception as exc:
                LOGGER.warning(
                    "repair_tests_loop broad_lookup_failed attempt=%s error=%s",
                    attempt,
                    exc,
                )

        combined_info_parts = [part for part in [targeted_info, broad_info] if part.strip()]
        combined_info = "\n\n".join(combined_info_parts)
        repair_context = build_repair_context(
            eval_result=eval_result,
            main_rs=current_test_rs,
            rustdoc_info=combined_info,
            problem_text=problem_text,
        )
        repair_context = truncate_feedback(repair_context, 12000)
        repair_prompt = REPAIR_PROMPT_TEMPLATE.format(context=repair_context)
        messages = [
            {"role": "system", "content": "You are an expert Rust unit-test repair assistant."},
            {"role": "user", "content": repair_prompt},
        ]
        response = openrouter_generate_code(messages)
        if response is None or not response.strip():
            LOGGER.warning("repair_tests_loop empty_repair_response attempt=%s", attempt)
            continue

        fixed = extract_rust_code_block(response)
        if not fixed:
            LOGGER.warning(
                "repair_tests_loop no_rust_block attempt=%s response=%s",
                attempt,
                preview_text(response, limit=500),
            )
            continue
        candidate = fixed.rstrip() + "\n"
        if candidate.strip() == previous_code.strip():
            same_streak += 1
        else:
            same_streak = 0
        previous_code = candidate
        current_test_rs = candidate

        if same_streak >= 2:
            LOGGER.warning("repair_tests_loop no_progress_break attempt=%s", attempt)
            break

    if best_test_rs.strip():
        LOGGER.warning(
            "repair_tests_loop exhausted_attempts returning_best best_score=%s",
            best_score,
        )
        return best_test_rs.rstrip() + "\n", best_eval

    raise RuntimeError("repair_tests_loop failed to produce any candidate test code")


def process_pair(
    problem_text: str,
    initial_test_rs: str,
    max_repair_attempts: int,
    eval_base: str,
    rustdocs_base: str,
    client: httpx.Client,
    fixed_deps: str,
) -> AlignmentResult:
    run_id = uuid.uuid4().hex[:8]
    updated_problem, updated_tests, skip_reason = review_problem_and_tests(
        problem_text=problem_text,
        test_rs=initial_test_rs,
        run_id=run_id,
    )
    if skip_reason:
        return AlignmentResult(
            problem_md=problem_text,
            test_rs=initial_test_rs,
            last_eval={"skip_reason": skip_reason},
            skipped=True,
        )

    if updated_problem is None or updated_tests is None:
        return AlignmentResult(
            problem_md=problem_text,
            test_rs=initial_test_rs,
            last_eval={"skip_reason": "could not parse response"},
            skipped=True,
        )

    stub_solution = generate_stub_solution(updated_problem)
    verified_test_rs, last_eval = repair_tests_loop(
        problem_text=updated_problem,
        test_rs=updated_tests,
        stub_solution=stub_solution,
        max_repair_attempts=max_repair_attempts,
        eval_base=eval_base,
        rustdocs_base=rustdocs_base,
        client=client,
        fixed_deps=fixed_deps,
    )
    return AlignmentResult(
        problem_md=updated_problem,
        test_rs=verified_test_rs,
        last_eval=last_eval,
        skipped=False,
    )


def process_staging_folder(
    input_dir: Path,
    output_dir: Path,
    max_repair_attempts: int,
    overwrite: bool,
) -> None:
    problems_in = input_dir / "problems"
    tests_in = input_dir / "tests"
    if not problems_in.is_dir() or not tests_in.is_dir():
        raise ValueError(f"input_dir must contain problems/ and tests/ subdirs: {input_dir}")

    problems_out = output_dir / "problems"
    tests_out = output_dir / "tests"
    problems_out.mkdir(parents=True, exist_ok=True)
    tests_out.mkdir(parents=True, exist_ok=True)
    manifest_path = output_dir / "manifest.jsonl"

    eval_base = env("RUST_EVAL_BASE_URL", "http://127.0.0.1:3002")
    rustdocs_base = env("RUSTDOCS_BASE_URL", "http://127.0.0.1:3001")

    with httpx.Client(timeout=120.0) as client:
        eval_server_warmup(eval_base, client)
        for md_path in sorted(problems_in.glob("*.md")):
            problem_id = md_path.stem
            test_path = tests_in / f"{problem_id}.rs"
            out_problem = problems_out / md_path.name
            out_test = tests_out / f"{problem_id}.rs"

            if out_problem.exists() and out_test.exists() and not overwrite:
                LOGGER.info("process_staging_folder skip id=%s reason=exists", problem_id)
                continue

            if not test_path.exists():
                LOGGER.warning("process_staging_folder skip id=%s reason=missing_test", problem_id)
                with manifest_path.open("a", encoding="utf-8") as mf:
                    mf.write(
                        json.dumps(
                            {
                                "id": problem_id,
                                "ok": False,
                                "skipped": False,
                                "error": f"missing test file: {test_path}",
                            }
                        )
                        + "\n"
                    )
                continue

            ok = False
            skipped = False
            error: Optional[str] = None
            try:
                problem_text = md_path.read_text(encoding="utf-8")
                initial_test_rs = test_path.read_text(encoding="utf-8")
                result = process_pair(
                    problem_text=problem_text,
                    initial_test_rs=initial_test_rs,
                    max_repair_attempts=max_repair_attempts,
                    eval_base=eval_base,
                    rustdocs_base=rustdocs_base,
                    client=client,
                    fixed_deps=FIXED_DEPENDENCIES,
                )

                out_problem.write_text(result.problem_md, encoding="utf-8")
                out_test.write_text(result.test_rs, encoding="utf-8")
                skipped = result.skipped
                ok = (result.last_eval.get("ok") is True) if isinstance(result.last_eval, dict) else False
                if skipped:
                    error = str(result.last_eval.get("skip_reason", "skipped")).strip()
            except Exception as exc:
                error = f"{type(exc).__name__}: {exc}"
                LOGGER.exception("process_staging_folder failed id=%s error=%s", problem_id, error)
                ok = False
                skipped = False

            with manifest_path.open("a", encoding="utf-8") as mf:
                mf.write(
                    json.dumps(
                        {
                            "id": problem_id,
                            "ok": ok,
                            "skipped": skipped,
                            "error": error,
                        }
                    )
                    + "\n"
                )


if __name__ == "__main__":
    configure_logging()
    parser = argparse.ArgumentParser(description="Review problem-test alignment and compile-verify tests.")
    parser.add_argument(
        "--input",
        required=True,
        help="Path to input folder containing problems/ and tests/ subdirectories.",
    )
    parser.add_argument(
        "--output-dir",
        default="./alignment_review_out",
        help="Directory to write reviewed problems/tests outputs.",
    )
    parser.add_argument(
        "--max-repair-attempts",
        type=int,
        default=6,
        help="Max repair-loop iterations per problem-test pair.",
    )
    parser.add_argument(
        "--overwrite",
        action="store_true",
        help="Re-process even if output files already exist.",
    )
    args = parser.parse_args()

    process_staging_folder(
        input_dir=Path(args.input),
        output_dir=Path(args.output_dir),
        max_repair_attempts=args.max_repair_attempts,
        overwrite=args.overwrite,
    )
