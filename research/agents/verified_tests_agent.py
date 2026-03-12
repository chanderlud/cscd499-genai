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
from langchain_ollama import ChatOllama
from langgraph.prebuilt import create_react_agent
from langchain_core.tools import tool

from helpers import *


FIXED_DEPENDENCIES = open("../rust_dependencies.md", "r").read()

RUST_FENCE_RE = re.compile(
    r"```(?:rust|rs)\s*(?P<code>[\s\S]*?)\s*```",
    re.IGNORECASE,
)


@dataclass
class TestImproveResult:
    test_rs: str
    last_eval: Dict[str, Any]
    skipped: bool


def build_test_agent(tools):
    model_name = os.getenv("OLLAMA_MODEL", "glm-4.7-flash:latest")
    model_url = os.getenv("OLLAMA_BASE_URL", "http://127.0.0.1:11434")

    model = ChatOllama(
        model=model_name,
        base_url=model_url,
        temperature=0,
        num_predict=int(os.getenv("OLLAMA_NUM_PREDICT", "8000")),
    )
    context_limit = int(os.getenv("OLLAMA_CONTEXT_TOKENS", "6000"))

    system_prompt = """You improve Rust unit tests for Win32/Windows API programming problems.

Workflow (follow in order):
- Step 1: First decide whether tests directly reference a Windows API type or constant in an assertion. Only if yes, call rust_win_search for those symbols to confirm correct windows crate import paths. Otherwise, do not add any use windows::... lines.
- Step 2: Rewrite/improve the test module using the provided current test module context.
- Step 3: Call evaluate_rust with the improved test module.
- Step 4: Iterate on compile/test errors until evaluate_rust reports ok=true.
- Step 5: Call final_answer with the final improved test module.

Hard rules:
- Tests must be inside: #[cfg(test)] mod tests { use super::*; ... }
- Test names must be descriptive, e.g. test_sha256_known_abc_vector.
- Never implement the function under test. Import from super::* only.
- Do NOT add use windows::... imports to the test module unless a Windows API type or constant is directly referenced in a test assertion (e.g., as an argument to assert_eq! or as a literal value). Never import Windows API symbols that are only needed by the implementation under super::*.
- Prefer deterministic assertions with known expected values.
- Do NOT call evaluate_rust again with identical code after a failure.
- If code_help is needed, pass problem_text and doc_results.
- Names should follow test_<function>_<scenario>_<expected_outcome>.
- Keep tests deterministic and fast; avoid random, UI interaction, and unbounded waits.
- Tests must complete quickly (under 5 seconds total).
- Cover error paths when implied by the spec.
- If writing meaningful, deterministic tests for this problem requires calling Windows API functions directly inside the test body (not via super::*), output SKIP: tests require direct Windows API calls and call final_answer with that skip message instead of generating tests.
"""

    LOGGER.info(
        "build_test_agent model=%s base_url=%s tool_count=%s",
        model_name,
        model_url,
        len(tools),
    )

    agent = create_react_agent(
        model=model,
        tools=tools,
        prompt=system_prompt,
    )

    return agent.with_config({"recursion_limit": 40}), model, context_limit


def build_test_tools(
    eval_state: Dict[str, Any],
    stub_solution: str,
    fixed_dependencies: str = FIXED_DEPENDENCIES,
) -> Tuple[list, dict]:
    base_tools, _ = build_tools("", fixed_dependencies, eval_state, run_tests=True)
    eval_base = env("RUST_EVAL_BASE_URL", "http://127.0.0.1:3002")
    client = httpx.Client(timeout=60.0)
    stub_with_main = ensure_empty_main(stub_solution)

    @tool("evaluate_rust")
    def evaluate_rust(test_rs: str) -> str:
        """
        Build + clippy + run tests for improved test modules using a prepended stub solution.
        """
        test_rs = normalize_rust_text(test_rs, field_name="test_rs")
        started = time.perf_counter()
        LOGGER.info(
            "test_evaluate_rust request test_rs_len=%s stub_len=%s",
            len(test_rs),
            len(stub_with_main),
        )
        try:
            full_main = stub_with_main.rstrip() + "\n\n" + test_rs.strip() + "\n"
            r = client.post(
                f"{eval_base}/evaluate",
                json={
                    "main_rs": full_main,
                    "dependencies": fixed_dependencies,
                    "run_tests": False,
                    "compile_tests": True,
                },
            )
            r.raise_for_status()
            data = r.json()
            duration_ms = int((time.perf_counter() - started) * 1000)
            LOGGER.info(
                "test_evaluate_rust ok duration_ms=%s summary=%s",
                duration_ms,
                summarize_tool_output("evaluate_rust", json.dumps(data, ensure_ascii=False)),
            )
            eval_state["last"] = data
            repair_message = build_repair_message(data, full_main)
            print(repair_message)
            return repair_message
        except Exception as exc:
            duration_ms = int((time.perf_counter() - started) * 1000)
            LOGGER.exception(
                "test_evaluate_rust error duration_ms=%s test_rs_len=%s",
                duration_ms,
                len(test_rs),
            )
            return f"evaluate_rust error: {type(exc).__name__}: {exc}"

    tools = []
    for t in base_tools:
        if t.name == "evaluate_rust":
            tools.append(evaluate_rust)
        else:
            tools.append(t)
    return tools, eval_state


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
    stub = match.group("code").strip() if match else answer.strip()
    return ensure_empty_main(stub.rstrip() + "\n")


def generate_improved_tests(
    problem_text: str,
    current_test_rs: str,
    feedback: str,
    run_id: str,
) -> Tuple[Optional[str], Optional[str]]:
    prompt = f"""You are an expert Rust test engineer for Windows API code.

Generate an improved Rust test module from the inputs below.

Rules:
- Output only one complete #[cfg(test)] mod tests {{ use super::*; ... }} block.
- Test names must be self-documenting and descriptive.
- Cover happy path, edge cases (empty input, boundary values), and error cases.
- Prefer deterministic assertions with known expected values.
- Avoid non-deterministic behavior (random, time-dependent, UI-dependent, process-spawning tests).
- Do not include any imports for items not used in the unit tests.
- Do NOT add use windows::... import lines unless a Windows API constant or type literal (e.g., FOLDERID_Desktop, GUID) appears directly in a test assertion. Never import Windows API symbols that are only consumed by the implementation.
- The final test environment has access to the full windows API, but tests should NOT import or use Windows API types directly. All Windows API interaction must go through the function under test via super::*. Only import from windows:: if a Windows API constant or type is used directly in a test assertion value.
- You have access to the tempfile, sha2, md5, rand, and regex crates in addition to the windows crate.
- Do not use #[ignore].
- If the problem involves GUI, message loops, interactive dialogs, tray icons, or non-deterministic system state that cannot be reliably tested in an automated windows test environment, or if writing any meaningful test requires calling Windows API functions directly inside the test body (not through super::*), output exactly:
SKIP: <one-line reason>

## Problem Markdown
{problem_text}

## Current Test Module
```rust
{current_test_rs}
```

## Compiler/Test Feedback
{feedback or "(none)"}
"""
    answer = code_help_tool(prompt, run_id)
    if not answer or not answer.strip():
        return None, "empty OpenRouter response"
    text = answer.strip()
    if text.startswith("SKIP:"):
        return None, text[len("SKIP:") :].strip() or "model requested skip"
    match = RUST_FENCE_RE.search(text)
    if match:
        return match.group("code").strip() + "\n", None
    if text.startswith("#[cfg(test)]") or text.startswith("#[cfg(all(test, windows))]"):
        return text.rstrip() + "\n", None
    return None, "could not extract Rust test module"


def improve_tests(
    problem_text: str,
    initial_test_rs: str,
    max_attempts: int = 6,
) -> TestImproveResult:
    run_id = uuid.uuid4().hex[:8]
    eval_state: Dict[str, Any] = {}
    feedback = ""
    last_eval: Dict[str, Any] = {}
    openrouter_same_streak = 0

    stub_solution = generate_stub_solution(problem_text)
    improved_test_rs, skip_reason = generate_improved_tests(
        problem_text=problem_text,
        current_test_rs=initial_test_rs,
        feedback=feedback,
        run_id=f"{run_id}-seed",
    )
    if improved_test_rs is None:
        LOGGER.warning("improve_tests skipped run_id=%s reason=%s", run_id, skip_reason)
        return TestImproveResult(test_rs=initial_test_rs, last_eval={}, skipped=True)
    print(improved_test_rs)

    tools, _ = build_test_tools(eval_state=eval_state, stub_solution=stub_solution)
    tool_map = {t.name: t for t in tools}
    agent, _, context_limit = build_test_agent(tools)

    current_test_rs = improved_test_rs

    for attempt in range(1, max_attempts + 1):
        LOGGER.info(
            "improve_tests attempt_start run_id=%s attempt=%s feedback_len=%s",
            run_id,
            attempt,
            len(feedback),
        )
        run_input = (
            "Improve this Rust test module and iterate until evaluate_rust passes.\n\n"
            "## Problem\n"
            f"{problem_text}\n\n"
            "## Current Test Module\n"
            "```rust\n"
            f"{current_test_rs}\n"
            "```\n\n"
            "## Feedback\n"
            f"{truncate_feedback(feedback, 4000) if feedback else '(none)'}\n"
        )
        messages = [{"role": "user", "content": run_input}]
        messages = compress_old_tool_messages(messages, keep_last_n=1)
        messages = apply_context_window(messages, max_tokens=context_limit)
        eval_state.clear()

        final_test_rs: Optional[str] = None
        try:
            result = agent.invoke({"messages": messages})
        except FinalAnswerException as answer:
            final_test_rs = answer.main_rs
        else:
            msgs = result.get("messages") or []
            salvaged = extract_rust_from_messages(msgs)
            if salvaged:
                final_test_rs = salvaged
            elif eval_state.get("last", {}):
                feedback = build_repair_message(
                    eval_state["last"],
                    current_test_rs,
                    problem_text=problem_text,
                )
                improved_next, reason = generate_improved_tests(
                    problem_text=problem_text,
                    current_test_rs=current_test_rs,
                    feedback=feedback,
                    run_id=f"{run_id}-attempt-{attempt}",
                )
                if improved_next is None:
                    LOGGER.warning("improve_tests skipped run_id=%s reason=%s", run_id, reason)
                    return TestImproveResult(test_rs=initial_test_rs, last_eval={}, skipped=True)
                if improved_next.strip() == current_test_rs.strip():
                    openrouter_same_streak += 1
                    if openrouter_same_streak >= 2:
                        LOGGER.warning("improve_tests gave up run_id=%s no_progress=2", run_id)
                        return TestImproveResult(test_rs=initial_test_rs, last_eval={}, skipped=True)
                else:
                    openrouter_same_streak = 0
                current_test_rs = improved_next
                continue
            else:
                raise RuntimeError("Agent did not produce a final improved test module.")

        if final_test_rs is None:
            raise RuntimeError("Agent finished without calling final_answer.")

        formatted = tool_map["format_rust"].invoke({"snippet": final_test_rs})
        if isinstance(formatted, str) and not (
            formatted.startswith("format_rust error") or formatted.startswith("format_rust failed")
        ):
            final_test_rs = formatted

        last_eval = eval_state.get("last", {})
        if not last_eval:
            tool_map["evaluate_rust"].invoke({"test_rs": final_test_rs})
            last_eval = eval_state.get("last", {})

        if last_eval.get("ok") is True:
            last_eval["_attempts"] = attempt
            return TestImproveResult(
                test_rs=final_test_rs.rstrip() + "\n",
                last_eval=last_eval,
                skipped=False,
            )

        feedback = build_repair_message(last_eval, final_test_rs, problem_text=problem_text)
        improved_next, reason = generate_improved_tests(
            problem_text=problem_text,
            current_test_rs=final_test_rs,
            feedback=feedback,
            run_id=f"{run_id}-attempt-{attempt}",
        )
        if improved_next is None:
            LOGGER.warning("improve_tests skipped run_id=%s reason=%s", run_id, reason)
            return TestImproveResult(test_rs=initial_test_rs, last_eval={}, skipped=True)
        if improved_next.strip() == current_test_rs.strip():
            openrouter_same_streak += 1
            if openrouter_same_streak >= 2:
                LOGGER.warning("improve_tests gave up run_id=%s no_progress=2", run_id)
                return TestImproveResult(test_rs=initial_test_rs, last_eval={}, skipped=True)
        else:
            openrouter_same_streak = 0
        current_test_rs = improved_next

    raise RuntimeError(
        f"Failed to improve tests within {max_attempts} attempts. "
        f"Last eval:\n{summarize_eval(last_eval)}"
    )


def process_staging_folder(
    input_dir: Path,
    output_dir: Path,
    max_attempts: int = 6,
    overwrite: bool = False,
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

    md_files = sorted(problems_in.glob("*.md"))
    LOGGER.info(
        "process_staging_folder start input=%s output=%s count=%s max_attempts=%s overwrite=%s",
        input_dir,
        output_dir,
        len(md_files),
        max_attempts,
        overwrite,
    )

    for md_path in md_files:
        problem_id = md_path.stem
        test_path = tests_in / f"{problem_id}.rs"
        out_problem = problems_out / md_path.name
        out_test = tests_out / f"{problem_id}.rs"

        if out_test.exists() and not overwrite:
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
                            "attempts": 0,
                            "error": f"missing test file: {test_path}",
                        }
                    )
                    + "\n"
                )
            continue

        attempts = 0
        error: Optional[str] = None
        ok = False
        skipped = False

        try:
            problem_text = md_path.read_text(encoding="utf-8")
            initial_test_rs = test_path.read_text(encoding="utf-8")
            result = improve_tests(problem_text, initial_test_rs, max_attempts=max_attempts)
            attempts = int(result.last_eval.get("_attempts", 0)) if result.last_eval else 0
            skipped = result.skipped
            if skipped:
                reason = str(result.last_eval.get("skip_reason", "non-deterministic or no progress")).strip()
                out_problem.write_text(problem_text, encoding="utf-8")
                out_test.write_text(
                    f"# SKIPPED: {reason}\n\n{initial_test_rs}",
                    encoding="utf-8",
                )
                LOGGER.warning("process_staging_folder skipped id=%s reason=%s", problem_id, reason)
                ok = False
            else:
                out_problem.write_text(problem_text, encoding="utf-8")
                out_test.write_text(result.test_rs, encoding="utf-8")
                ok = True
        except Exception as exc:
            attempts = attempts or max_attempts
            error = f"{type(exc).__name__}: {exc}"
            LOGGER.exception("process_staging_folder failed id=%s error=%s", problem_id, error)
            ok = False

        with manifest_path.open("a", encoding="utf-8") as mf:
            mf.write(
                json.dumps(
                    {
                        "id": problem_id,
                        "ok": ok,
                        "skipped": skipped,
                        "attempts": attempts,
                        "error": error,
                    }
                )
                + "\n"
            )


if __name__ == "__main__":
    configure_logging()
    parser = argparse.ArgumentParser(description="Improve staged Rust tests with an agent loop.")
    parser.add_argument(
        "--input",
        required=True,
        help="Path to staging folder with problems/ and tests/ subdirectories.",
    )
    parser.add_argument(
        "--output-dir",
        default="./verified_tests_out",
        help="Directory to write improved problems/tests outputs.",
    )
    parser.add_argument(
        "--max-attempts",
        type=int,
        default=6,
        help="Max agent iterations per problem.",
    )
    parser.add_argument(
        "--overwrite",
        action="store_true",
        help="Re-process even if output test already exists.",
    )
    args = parser.parse_args()

    process_staging_folder(
        input_dir=Path(args.input),
        output_dir=Path(args.output_dir),
        max_attempts=args.max_attempts,
        overwrite=args.overwrite,
    )
