import argparse
import concurrent.futures
import os
import re
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, Optional

import httpx

from helpers import (
    LOGGER,
    StepRecorder,
    build_repair_context,
    configure_logging,
    env,
    eval_server_evaluate,
    eval_server_format,
    eval_server_warmup,
    extract_rust_code_block,
    openrouter_generate_code,
    preview_text,
)


FIXED_DEPENDENCIES = (
    Path(__file__).resolve().parent.parent / "rust_dependencies.md"
).read_text(encoding="utf-8")

SYSTEM_PROMPT = """You are an expert Rust engineer focused on lint quality and correctness.

Task:
- Fix all Clippy warnings and errors in the provided Rust file.

Output requirements:
- Output exactly one complete Rust file inside a single ```rust code fence.
- No explanation text outside the fence.
- Use only stable Rust.

Hard constraints:
- Do NOT add lint-suppression attributes such as:
  - #[allow(...)]
  - #![allow(...)]
  - #[warn(...)]
  - #![warn(...)]
  - #[deny(...)]
  - #![deny(...)]
  - #[clippy::...]
  - #![clippy::...]
- You must fix root causes instead of suppressing diagnostics.
- Preserve existing behavior and logic; change only what is needed to resolve diagnostics.
"""

REPAIR_PROMPT_TEMPLATE = """Your previous code attempt still had build/clippy issues. Fix ONLY the reported issues.
Do NOT rewrite from scratch; make targeted repairs.

{context}

Repair constraints:
- Resolve all clippy warnings and errors completely.
- Do NOT add #[allow(...)], #![allow(...)], #[warn(...)], #[deny(...)], or #[clippy::...] style suppression attributes.
- Fix root causes and preserve existing behavior.

Output the complete fixed file in a single ```rust code fence.
"""


@dataclass
class ClippyFixResult:
    main_rs: str
    last_eval: Dict[str, Any]
    verified: bool = True


def _clippy_warning_count(eval_result: Dict[str, Any]) -> int:
    clippy = eval_result.get("clippy") if isinstance(eval_result.get("clippy"), dict) else {}
    diagnostics = clippy.get("diagnostics") if isinstance(clippy.get("diagnostics"), dict) else {}
    errors = int(diagnostics.get("errors", 0) or 0)
    warnings = int(diagnostics.get("warnings", 0) or 0)
    build = eval_result.get("build") if isinstance(eval_result.get("build"), dict) else {}
    build_errors = int((build.get("diagnostics") or {}).get("errors", 0) or 0)
    return errors + warnings + build_errors


_ALLOW_ATTR_RE = re.compile(
    r"#\s*!?\s*\[\s*(allow|clippy\s*::\s*allow)\s*\(",
    re.IGNORECASE,
)


def _contains_allow_attribute(code: str) -> bool:
    return bool(_ALLOW_ATTR_RE.search(code))


def fix_clippy(
    source_code: str,
    max_attempts: int = 8,
    output_dir: Optional[Path] = None,
    file_id: Optional[str] = None,
    resume: bool = False,
) -> ClippyFixResult:
    _ = resume
    run_id = uuid.uuid4().hex[:8]
    eval_base = env("RUST_EVAL_BASE_URL", "http://127.0.0.1:3002")
    recorder = StepRecorder(run_id=run_id, output_dir=output_dir, problem_id=file_id)

    LOGGER.info(
        "fix_clippy start run_id=%s attempts=%s source_len=%s file_id=%s",
        run_id,
        max_attempts,
        len(source_code),
        file_id,
    )

    best_code = ""
    best_eval: Dict[str, Any] = {}
    best_score = 10**9
    previous_code = ""
    same_streak = 0
    repair_context = ""

    with httpx.Client(timeout=120.0) as client:
        eval_server_warmup(eval_base, client)

        pre_eval = eval_server_evaluate(
            main_rs=source_code,
            unit_tests_private="",
            fixed_deps=FIXED_DEPENDENCIES,
            eval_base=eval_base,
            client=client,
            run_tests=False,
        )
        recorder.record_step(
            attempt=0,
            step_type="pre_check",
            code=source_code,
            eval_result=pre_eval,
            extra_context={},
        )
        score = _clippy_warning_count(pre_eval)
        pre_build = pre_eval.get("build") if isinstance(pre_eval.get("build"), dict) else {}
        if score == 0 and pre_build.get("ok") is True:
            LOGGER.info("fix_clippy no_warnings_found short_circuit run_id=%s", run_id)
            return ClippyFixResult(main_rs=source_code.rstrip() + "\n", last_eval=pre_eval)
        repair_context = build_repair_context(
            eval_result=pre_eval,
            main_rs=source_code,
            rustdoc_info="",
            problem_text=source_code,
        )

        for attempt in range(1, max_attempts + 1):
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
                    "## Build/Clippy Results\n"
                    "No Rust code block was generated in the previous attempt.\n\n"
                    "## Repair Instructions\n"
                    "- Output the complete fixed file in a single ```rust code fence.\n"
                    "- Do not use lint-suppression attributes.\n"
                )
                continue

            if _contains_allow_attribute(code):
                recorder.record_step(
                    attempt=attempt,
                    step_type="cheat_detected",
                    code=code,
                    eval_result=None,
                    extra_context={"reason": "forbidden allow attribute detected"},
                )
                repair_context = (
                    "## Build/Clippy Results\n"
                    "Forbidden lint suppression detected: #[allow(...)] or #![allow(...)].\n\n"
                    "## Repair Instructions\n"
                    "- Remove all allow-based suppression attributes.\n"
                    "- Fix the root causes of diagnostics instead.\n"
                    "- Output the complete file in a single ```rust code fence.\n"
                )
                continue

            recorder.record_step(
                attempt=attempt,
                step_type="generate",
                code=code,
                eval_result=None,
                extra_context={"phase": "initial" if attempt == 1 else "repair"},
            )

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
                    "## Build/Clippy Results\n"
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
                extra_context={},
            )

            score = _clippy_warning_count(eval_result)
            if score < best_score:
                best_score = score
                best_code = code
                best_eval = eval_result

            build = eval_result.get("build") if isinstance(eval_result.get("build"), dict) else {}
            if score == 0 and build.get("ok") is True:
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
                            unit_tests_private="",
                            fixed_deps=FIXED_DEPENDENCIES,
                            eval_base=eval_base,
                            client=client,
                            run_tests=False,
                        )
                    except Exception as exc:
                        LOGGER.warning("attempt=%s formatted_recheck_failed error=%s", attempt, exc)
                        formatted_eval = None

                    formatted_build = (
                        formatted_eval.get("build")
                        if isinstance(formatted_eval, dict) and isinstance(formatted_eval.get("build"), dict)
                        else {}
                    )
                    if (
                        isinstance(formatted_eval, dict)
                        and _clippy_warning_count(formatted_eval) == 0
                        and formatted_build.get("ok") is True
                        and not _contains_allow_attribute(formatted)
                    ):
                        recorder.record_final(formatted, formatted_eval)
                        return ClippyFixResult(main_rs=formatted.rstrip() + "\n", last_eval=formatted_eval)

                    recorder.record_final(code, eval_result)
                    return ClippyFixResult(main_rs=code.rstrip() + "\n", last_eval=eval_result)

                recorder.record_final(code, eval_result)
                return ClippyFixResult(main_rs=code.rstrip() + "\n", last_eval=eval_result)

            repair_context = build_repair_context(
                eval_result=eval_result,
                main_rs=code,
                rustdoc_info="",
                problem_text=source_code,
            )

            if code.strip() == previous_code.strip():
                same_streak += 1
            else:
                same_streak = 0
            previous_code = code

            if same_streak >= 2:
                repair_context += (
                    "\n\nWARNING: Your previous repair attempt returned identical code. "
                    "You MUST make a different change this time. Focus on remaining clippy/build diagnostics."
                )

    if best_code.strip():
        LOGGER.warning(
            "fix_clippy exhausted_attempts returning_best run_id=%s best_score=%s",
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
        return ClippyFixResult(main_rs=best_code.rstrip() + "\n", last_eval=best_eval, verified=False)

    raise RuntimeError(f"Failed to fix within {max_attempts} attempts and no valid code was produced.")


def process_input_directory(
    input_dir: Path,
    output_dir: Path,
    max_attempts: int,
    overwrite: bool,
    resume: bool = False,
    workers: int = 5,
) -> None:
    if not input_dir.is_dir():
        raise ValueError(f"Input directory not found: {input_dir}")

    output_dir.mkdir(parents=True, exist_ok=True)
    input_files = sorted(input_dir.glob("*.rs"))
    LOGGER.info(
        "process_input_directory start input=%s count=%s output=%s max_attempts=%s overwrite=%s workers=%s",
        input_dir,
        len(input_files),
        output_dir,
        max_attempts,
        overwrite,
        workers,
    )

    futures: Dict[concurrent.futures.Future[ClippyFixResult], tuple[str, Path]] = {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as executor:
        for rs_path in input_files:
            file_id = rs_path.stem
            out_path = output_dir / f"{file_id}.rs"

            if out_path.exists() and not overwrite:
                LOGGER.info("process_input_directory skip id=%s reason=exists", file_id)
                continue

            source_code = rs_path.read_text(encoding="utf-8")
            future = executor.submit(
                fix_clippy,
                source_code=source_code,
                max_attempts=max_attempts,
                output_dir=Path("out"),
                file_id=file_id,
                resume=resume,
            )
            futures[future] = (file_id, out_path)

        for future in concurrent.futures.as_completed(futures):
            file_id, out_path = futures[future]
            try:
                result = future.result()
                if result.verified:
                    out_path.write_text(result.main_rs, encoding="utf-8")
                    LOGGER.info(
                        "process_input_directory ok id=%s clippy_score=%s",
                        file_id,
                        _clippy_warning_count(result.last_eval),
                    )
                else:
                    LOGGER.warning(
                        "process_input_directory best_effort_skipped id=%s best_score=%s steps_dir=out/steps/%s",
                        file_id,
                        _clippy_warning_count(result.last_eval),
                        file_id,
                    )
            except Exception as exc:
                LOGGER.exception("process_input_directory failed id=%s error=%s", file_id, exc)
                continue


if __name__ == "__main__":
    configure_logging()
    parser = argparse.ArgumentParser(description="Clippy warning fix agent")
    parser.add_argument("--input-dir", required=True, help="Directory of .rs files to fix.")
    parser.add_argument("--output-dir", required=True, help="Directory to write fixed .rs files.")
    parser.add_argument("--max-attempts", type=int, default=8)
    parser.add_argument("--overwrite", action="store_true")
    parser.add_argument("--resume", action="store_true")
    parser.add_argument("--workers", type=int, default=5)
    args = parser.parse_args()

    process_input_directory(
        input_dir=Path(args.input_dir),
        output_dir=Path(args.output_dir),
        max_attempts=args.max_attempts,
        overwrite=args.overwrite,
        resume=args.resume,
        workers=args.workers,
    )
