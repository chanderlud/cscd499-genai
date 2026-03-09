import argparse
import json
import os
import re
import time
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, List, Optional, Set, Tuple

import httpx
from langchain_core.messages import HumanMessage
from langchain_ollama import ChatOllama
from langgraph.prebuilt import create_react_agent

from helpers import (
    LOGGER,
    FinalAnswerException,
    apply_context_window,
    build_repair_message,
    build_tools,
    code_help_tool,
    compress_old_tool_messages,
    configure_logging,
    extract_message_text,
    preview_text,
    refactor_with_specialist,
    summarize_eval,
    truncate_feedback,
)


FIXED_DEPENDENCIES = open("../rust_dependencies.md", "r").read()
_HTTPX_READY = httpx is not None


@dataclass
class SnippetResult:
    idea: str
    main_rs: str
    last_eval: Dict[str, Any]


def _windows_features_summary(dependencies_text: str) -> str:
    features = re.findall(r'"([^"]+)"', dependencies_text)
    if not features:
        return "windows default feature set from rust_dependencies.md"
    if len(features) <= 20:
        return ", ".join(features)
    return ", ".join(features[:20]) + f", ... (+{len(features) - 20} more)"


def generate_snippet_ideas(sample_code: str, count: int, already_produced: Set[str]) -> List[str]:
    run_id = uuid.uuid4().hex[:8]
    already_produced_list = sorted(already_produced)
    features_summary = _windows_features_summary(FIXED_DEPENDENCIES)

    prompt = f"""You are an expert Windows API programmer in Rust.

Below is a Rust code sample that uses the `windows` or `winapi` crate.

Your task: Identify {count} DISTINCT, concrete Windows API usage ideas that can be derived from or inspired by this sample. Each idea must:
1. Be a self-contained, standalone Rust snippet (no external state, no UI interaction).
2. Use the `windows` crate (NOT winapi).
3. Demonstrate a specific Win32 API call or pattern (e.g., registry read, process enumeration, file attribute query, service status check, etc.).
4. Be different from every idea in this already-produced list: {already_produced_list}.
5. Be implementable with the available windows crate features: {features_summary}.

For each idea, output a single line in this exact format:
IDEA: <short imperative title, max 12 words>

Output exactly {count} IDEA: lines and nothing else.

## Sample Code
```rust
{sample_code}
```"""

    answer = code_help_tool(prompt, run_id)
    if not answer:
        LOGGER.warning("generate_snippet_ideas empty_response run_id=%s", run_id)
        return []

    found: List[str] = []
    for raw_line in answer.splitlines():
        line = raw_line.strip()
        if not line.startswith("IDEA:"):
            continue
        idea = line[len("IDEA:") :].strip()
        if not idea:
            continue
        if idea in already_produced:
            continue
        if idea in found:
            continue
        found.append(idea)
        if len(found) >= count:
            break

    LOGGER.info(
        "generate_snippet_ideas run_id=%s requested=%s returned=%s",
        run_id,
        count,
        len(found),
    )
    return found


def generate_snippet_idea_variants(idea: str) -> str:
    run_id = uuid.uuid4().hex[:8]
    prompt = f"""You are an expert Rust/Windows API programmer.

Idea: {idea}

Provide concise, concrete implementation notes for this idea as a standalone Rust snippet using the `windows` crate. Include:
1. The specific Win32 API functions to call (use W-suffix variants).
2. The exact windows crate module paths (e.g., windows::Win32::System::Registry::RegOpenKeyExW).
3. The correct error-handling pattern for each call.
4. Any important flags, structs, or constants needed.
5. The expected fn main() flow in 3-5 bullet points.

Be specific and brief. No code blocks."""
    guidance = code_help_tool(prompt, run_id)
    if not guidance:
        LOGGER.warning("generate_snippet_idea_variants empty_response run_id=%s idea=%r", run_id, idea)
        return ""
    return guidance.strip()


def build_snippet_agent(tools):
    model_name = os.getenv("OLLAMA_MODEL", "glm-4.7-flash:latest")
    model_url = os.getenv("OLLAMA_BASE_URL", "http://127.0.0.1:11434")

    model = ChatOllama(
        model=model_name,
        base_url=model_url,
        temperature=0,
        num_predict=int(os.getenv("OLLAMA_NUM_PREDICT", "8000")),
    )
    context_limit = int(os.getenv("OLLAMA_CONTEXT_TOKENS", "6000"))

    system_prompt = """You generate standalone, self-contained Rust code snippets that demonstrate Windows API usage via the `windows` crate.

Hard rules:
- Each snippet must be a complete, standalone Rust file: include all `use` imports at the top, a `fn main()` that exercises the API, and proper error handling.
- Do NOT write unit tests. Do NOT write library functions without a main().
- Use ms_doc_search and rust_win_search to confirm API signatures and correct windows crate import paths.
- Use code_help when you need expert guidance on a specific implementation problem.
- Call evaluate_rust to compile + clippy check. Iterate until ok=true with zero clippy warnings.
- Do NOT call evaluate_rust again with identical code after a failure. Make a concrete fix first.
- Only call format_rust once evaluate_rust returns ok=true, then call final_answer with the formatted result.
- After evaluate_rust returns ok=true, call code_review with the snippet and the idea description.
- If code_review returns NEEDS_CHANGES or REJECT, fix all CRITICAL/MAJOR issues and re-evaluate.

Quality rules:
- Use the `windows` crate exclusively (not winapi).
- Prefer W-suffix variants of Win32 functions.
- Use `windows::core::{Result, Error}` for error propagation.
- Minimize `unsafe` blocks; justify each with a comment.
- Use `?` operator for Result-returning calls.
- For non-Result Win32 calls, check the return value and call `GetLastError` / `windows::core::Error::from_win32()`.
- Include `#[allow(unused_imports)] use windows::core::{Result, Error};` at the top.
- The snippet must compile and pass clippy with no warnings (deny(warnings) is enforced).

Wide string helper (use when needed):
```rust
fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}
```"""

    LOGGER.info(
        "build_snippet_agent model=%s base_url=%s tool_count=%s httpx_ready=%s",
        model_name,
        model_url,
        len(tools),
        _HTTPX_READY,
    )

    agent = create_react_agent(
        model=model,
        tools=tools,
        prompt=system_prompt,
    )

    return agent.with_config({"recursion_limit": 40}), model, context_limit


def build_snippet_tools(eval_state: Dict[str, Any]) -> Tuple[list, dict]:
    return build_tools("", FIXED_DEPENDENCIES, eval_state)


def generate_snippet(
    idea: str,
    sample_code: str,
    guidance: str = "",
    max_attempts: int = 6,
) -> SnippetResult:
    run_id = uuid.uuid4().hex[:8]
    eval_state: Dict[str, Any] = {}
    tools_list, _ = build_snippet_tools(eval_state)
    agent, agent_model, context_limit = build_snippet_agent(tools_list)
    tool_map = {t.name: t for t in tools_list}

    feedback = ""
    last_eval: Dict[str, Any] = {}
    refactor_done = False
    refactor_repair_rs: Optional[str] = None

    initial_prompt = f"""
Generate a standalone Rust snippet that demonstrates: {idea}

Requirements:
- Must be a complete src/main.rs file with fn main().
- Use only the `windows` crate (features already configured in Cargo.toml).
- Include all necessary `use` imports.
- Follow best practices: Result propagation, minimal unsafe, W-suffix functions.
- The snippet must compile and pass clippy with zero warnings.

Reference sample (for API context - do NOT copy verbatim, create a fresh implementation):
```rust
{sample_code}
```
"""
    if guidance.strip():
        initial_prompt += f"""

## Implementation Guidance
{guidance.strip()}
"""

    LOGGER.info(
        "generate_snippet start run_id=%s idea=%r sample_len=%s guidance_len=%s model=%s max_attempts=%s",
        run_id,
        idea,
        len(sample_code),
        len(guidance),
        getattr(agent_model, "model", "unknown"),
        max_attempts,
    )

    for attempt in range(1, max_attempts + 1):
        LOGGER.info(
            "generate_snippet attempt_start run_id=%s attempt=%s feedback_len=%s",
            run_id,
            attempt,
            len(feedback),
        )
        if refactor_repair_rs:
            run_input = (
                initial_prompt
                + "\n\n---\nREPAIR FEEDBACK:\n"
                + truncate_feedback(feedback, 3000)
                + "\n\n---\nREFACTORED CODE TO REPAIR (fix only the errors above, do not rewrite from scratch):\n```rust\n"
                + refactor_repair_rs
                + "\n```"
            )
            refactor_repair_rs = None
        elif feedback:
            run_input = initial_prompt + "\n\n---\nREPAIR FEEDBACK:\n" + truncate_feedback(feedback, 3000)
        else:
            run_input = initial_prompt

        messages = [{"role": "user", "content": run_input}]
        messages = compress_old_tool_messages(messages, keep_last_n=1)
        messages = apply_context_window(messages, max_tokens=context_limit)
        eval_state.clear()

        invoke_started = time.perf_counter()
        main_rs: Optional[str] = None
        try:
            result = agent.invoke({"messages": messages})
        except FinalAnswerException as answer:
            LOGGER.info(
                "generate_snippet final_answer_exception run_id=%s attempt=%s main_rs_len=%s",
                run_id,
                attempt,
                len(answer.main_rs),
            )
            main_rs = answer.main_rs
        else:
            duration_ms = int((time.perf_counter() - invoke_started) * 1000)
            msgs = result.get("messages") or []
            LOGGER.debug(
                "snippet_agent_invoke completed run_id=%s attempt=%s duration_ms=%s message_count=%s",
                run_id,
                attempt,
                duration_ms,
                len(msgs),
            )
            if main_rs is None:
                raise RuntimeError("Agent did not produce a final answer.")

        if main_rs is None:
            raise RuntimeError("Agent finished without calling final_answer.")

        LOGGER.info(
            "generate_snippet format_candidate run_id=%s attempt=%s main_rs_len=%s",
            run_id,
            attempt,
            len(main_rs),
        )
        formatted = tool_map["format_rust"].invoke({"snippet": main_rs})
        if isinstance(formatted, str) and not (
            formatted.startswith("format_rust error") or formatted.startswith("format_rust failed")
        ):
            LOGGER.info(
                "generate_snippet format_applied run_id=%s attempt=%s before_len=%s after_len=%s",
                run_id,
                attempt,
                len(main_rs),
                len(formatted),
            )
            main_rs = formatted
        else:
            LOGGER.warning(
                "generate_snippet format_skipped run_id=%s attempt=%s summary=%s",
                run_id,
                attempt,
                preview_text(formatted, limit=240),
            )

        last_eval = eval_state.get("last", {})
        if not last_eval:
            tool_map["evaluate_rust"].invoke({"main_rs": main_rs})
            last_eval = eval_state.get("last", {})

        if last_eval.get("ok") is True:
            LOGGER.info(
                "generate_snippet success run_id=%s attempt=%s summary=%s",
                run_id,
                attempt,
                preview_text(last_eval, limit=240),
            )
            if not refactor_done:
                main_rs = refactor_with_specialist(main_rs, idea, run_id)
                refactor_done = True
                eval_state.clear()
                tool_map["evaluate_rust"].invoke({"main_rs": main_rs})
                last_eval = eval_state.get("last", {})
                if last_eval.get("ok") is True:
                    return SnippetResult(
                        idea=idea,
                        main_rs=main_rs.rstrip() + "\n",
                        last_eval=last_eval,
                    )
                feedback = build_repair_message(last_eval, main_rs)
                refactor_repair_rs = main_rs
                continue
            return SnippetResult(
                idea=idea,
                main_rs=main_rs.rstrip() + "\n",
                last_eval=last_eval,
            )

        feedback = build_repair_message(last_eval, main_rs)
        LOGGER.warning(
            "generate_snippet attempt_failed run_id=%s attempt=%s feedback=%r",
            run_id,
            attempt,
            preview_text(feedback, limit=400),
        )

    LOGGER.error(
        "generate_snippet exhausted_attempts run_id=%s max_attempts=%s last_eval=%r",
        run_id,
        max_attempts,
        preview_text(last_eval, limit=400),
    )
    raise RuntimeError(
        f"Failed to generate snippet for idea {idea!r} within {max_attempts} attempts. "
        f"Last eval:\n{summarize_eval(last_eval)}"
    )


def produce_snippets(
    sample_code: str,
    ideas_per_sample: int = 5,
    max_attempts_per_idea: int = 6,
    output_dir: Optional[Path] = None,
    produced_ideas: Optional[Set[str]] = None,
) -> List[SnippetResult]:
    produced = produced_ideas if produced_ideas is not None else set()
    results: List[SnippetResult] = []

    ideas = generate_snippet_ideas(sample_code, count=ideas_per_sample, already_produced=produced)

    if output_dir:
        output_dir.mkdir(parents=True, exist_ok=True)

    for idea in ideas:
        if idea in produced:
            continue

        guidance = generate_snippet_idea_variants(idea)

        try:
            result = generate_snippet(idea, sample_code, guidance, max_attempts=max_attempts_per_idea)
            produced.add(idea)
            results.append(result)

            if output_dir:
                snippet_id = uuid.uuid4().hex[:8]
                out_path = output_dir / f"{snippet_id}.rs"
                out_path.write_text(result.main_rs, encoding="utf-8")

                manifest_path = output_dir / "manifest.jsonl"
                with manifest_path.open("a", encoding="utf-8") as f:
                    f.write(json.dumps({"id": snippet_id, "idea": idea, "ok": True}) + "\n")

            LOGGER.info("produce_snippets ok idea=%r snippet_len=%s", idea, len(result.main_rs))
        except Exception as exc:
            LOGGER.warning("produce_snippets failed idea=%r error=%s", idea, exc)
            if output_dir:
                manifest_path = output_dir / "manifest.jsonl"
                with manifest_path.open("a", encoding="utf-8") as f:
                    f.write(
                        json.dumps({"id": None, "idea": idea, "ok": False, "error": str(exc)})
                        + "\n"
                    )

    return results


def _collect_input_samples(input_path: Path) -> List[Path]:
    if input_path.is_file() and input_path.suffix.lower() == ".rs":
        return [input_path]
    if input_path.is_dir():
        return sorted(p for p in input_path.rglob("*.rs") if p.is_file())
    raise ValueError(f"Input must be a .rs file or directory of .rs files: {input_path}")


if __name__ == "__main__":
    configure_logging()

    parser = argparse.ArgumentParser(description="Generate validated Windows Rust snippets from sample code.")
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
        "--ideas-per-sample",
        type=int,
        default=5,
        help="Number of distinct ideas to generate per input sample.",
    )
    parser.add_argument(
        "--max-attempts",
        type=int,
        default=6,
        help="Max agent attempts per idea.",
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
    shared_produced_ideas: Set[str] = set()

    LOGGER.info(
        "windows_snippet_agent start input=%s samples=%s output_dir=%s ideas_per_sample=%s max_attempts=%s overwrite=%s",
        input_path,
        len(sample_paths),
        output_root,
        args.ideas_per_sample,
        args.max_attempts,
        args.overwrite,
    )

    for sample_path in sample_paths:
        sample_code = sample_path.read_text(encoding="utf-8")
        sample_output_dir = output_root / sample_path.stem

        if sample_output_dir.exists() and not args.overwrite:
            manifest_path = sample_output_dir / "manifest.jsonl"
            has_existing = manifest_path.exists() or any(sample_output_dir.glob("*.rs"))
            if has_existing:
                LOGGER.info("Skipping %s (output exists, use --overwrite to regenerate).", sample_path)
                continue

        results = produce_snippets(
            sample_code=sample_code,
            ideas_per_sample=args.ideas_per_sample,
            max_attempts_per_idea=args.max_attempts,
            output_dir=sample_output_dir,
            produced_ideas=shared_produced_ideas,
        )
        LOGGER.info("Completed sample=%s generated=%s snippets", sample_path, len(results))
