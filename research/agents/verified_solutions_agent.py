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

from langgraph.prebuilt import create_react_agent
from langchain_core.messages import HumanMessage, ToolMessage, trim_messages
from langchain_core.tools import tool
from langchain_ollama import ChatOllama

from helpers import *

FIXED_DEPENDENCIES = open("../rust_dependencies.md", "r").read()


class FinalAnswerException(Exception):
    def __init__(self, main_rs: str):
        super().__init__("final_answer")
        self.main_rs = main_rs


@dataclass
class SolveResult:
    main_rs: str
    last_eval: Dict[str, Any]


def build_tools(unit_tests_private: str, _eval_state: Optional[dict] = None):
    eval_state = _eval_state if _eval_state is not None else {}
    msdocs_base = env("MSDOCS_BASE_URL", "http://127.0.0.1:3000")
    rustdocs_base = env("RUSTDOCS_BASE_URL", "http://127.0.0.1:3001")
    eval_base = env("RUST_EVAL_BASE_URL", "http://127.0.0.1:3002")

    client = httpx.Client(timeout=60.0)
    try:
        warmup_resp = client.get(f"{eval_base}/warmup", timeout=600)
        warmup_resp.raise_for_status()
        LOGGER.info(
            "eval warmup ok response=%s",
            preview_text(warmup_resp.json(), limit=160),
        )
    except Exception as exc:
        LOGGER.warning("eval warmup failed error=%s", exc)

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
            preview_text(q, limit=120),
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
                preview_text(q, limit=80),
            )
            return json.dumps(data, ensure_ascii=False, indent=2)[:18000]
        except Exception as e:
            duration_ms = int((time.perf_counter() - started) * 1000)
            LOGGER.exception(
                "ms_doc_search failed duration_ms=%s q=%r",
                duration_ms,
                preview_text(q, limit=80),
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
            preview_text(q, limit=120),
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
                len(data) if isinstance(data, list) else len(data.get("results", [])) if isinstance(data,
                                                                                                    dict) else "n/a",
                preview_text(q, limit=80),
            )
            return json.dumps(data, ensure_ascii=False, indent=2)[:18000]
        except Exception as e:
            duration_ms = int((time.perf_counter() - started) * 1000)
            LOGGER.exception(
                "rust_win_search failed duration_ms=%s q=%r",
                duration_ms,
                preview_text(q, limit=80),
            )
            return f"rust_win_search error: {type(e).__name__}: {e}"

    @tool("format_rust")
    def format_rust(snippet: str) -> str:
        """
        Format Rust snippet using the formatting API. Returns formatted code (or error text).
        """
        snippet = normalize_rust_text(snippet, field_name="snippet")
        started = time.perf_counter()
        LOGGER.debug("format_rust request snippet_len=%s", len(snippet))
        try:
            r = client.post(f"{eval_base}/format", json={"snippet": snippet})
            r.raise_for_status()
            data = r.json()
            if data.get("ok") and data.get("formatted"):
                formatted = normalize_rust_text(data["formatted"], field_name="formatted")
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
                preview_text(data, limit=200),
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
        main_rs = normalize_rust_text(main_rs, field_name="main_rs")
        print(main_rs)
        started = time.perf_counter()
        LOGGER.info(
            "evaluate_rust request main_rs_len=%s hidden_tests_len=%s",
            len(main_rs),
            len(unit_tests_private),
        )
        try:
            full_main = (
                    ensure_empty_main(main_rs).rstrip()
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
                summarize_tool_output("evaluate_rust", json.dumps(data, ensure_ascii=False)),
            )
            eval_state["last"] = data
            repair_message = build_repair_message(data, full_main)
            print(repair_message)
            return repair_message
        except Exception as e:
            duration_ms = int((time.perf_counter() - started) * 1000)
            LOGGER.exception("evaluate_rust error duration_ms=%s main_rs_len=%s", duration_ms, len(main_rs))
            return f"evaluate_rust error: {type(e).__name__}: {e}"

    @tool("code_review")
    def code_review(main_rs: str, problem_text: str) -> str:
        """
        Request an expert code review from an external model before calling final_answer.
        Use this after evaluate_rust returns ok=true to catch logic errors, unsafe issues,
        and Clippy violations that the build harness may not surface.
        Pass the full src/main.rs content and the original problem statement.
        Returns structured review feedback with a VERDICT, ISSUES, and SUGGESTED_FIXES.
        """
        main_rs = normalize_rust_text(main_rs, field_name="main_rs")
        run_id = uuid.uuid4().hex[:8]

        prompt = (
            "You are an expert Rust code reviewer specializing in Windows API programming using the `windows` crate.\n\n"
            "## Task\n"
            "Review the Rust code below for correctness, safety, and idiomatic style. The code is a solution to the problem "
            "described in the \"Problem\" section. Hidden unit tests will be appended and run against this code.\n\n"
            "## Review Criteria (address each in order)\n"
            "1. **Correctness** — Does the logic correctly solve the stated problem? Are Win32 API calls used with the right "
            "arguments, flags, and error-checking patterns?\n"
            "2. **Safety** — Is `unsafe` minimized and justified? Are raw pointers, handles, and lifetimes managed correctly? "
            "Are resources (HANDLEs, allocations) properly released?\n"
            "3. **Error handling** — Are all fallible Win32 calls checked? Is `windows::core::Result` / `HRESULT` propagated correctly?\n"
            "4. **Idiomatic Rust** — Does the code follow Rust conventions (ownership, borrowing, naming)? Are there unnecessary "
            "clones, unwraps, or panics?\n"
            "5. **Clippy compliance** — Would `cargo clippy` flag anything? List specific lints if so.\n"
            "6. **Import completeness** — Are all `use` paths present and correct for the `windows` crate features used?\n\n"
            "## Output Format\n"
            "Respond with a structured review using these exact headings:\n"
            "### VERDICT\n"
            "One of: APPROVE | NEEDS_CHANGES | REJECT\n"
            "(APPROVE = ready to submit; NEEDS_CHANGES = fixable issues found; REJECT = fundamental logic error)\n\n"
            "### ISSUES\n"
            "A numbered list of concrete issues. For each issue state: severity (CRITICAL/MAJOR/MINOR), location (line or construct), "
            "and a specific fix recommendation. If none, write \"None.\"\n\n"
            "### SUGGESTED_FIXES\n"
            "For each CRITICAL or MAJOR issue, describe the exact change needed in plain English. Do NOT rewrite the entire file.\n\n"
            "### SUMMARY\n"
            "One paragraph summarizing overall quality and the most important action to take next.\n\n"
            "## Problem\n"
            f"{problem_text}\n\n"
            "## Code Under Review\n"
            "```rust\n"
            f"{main_rs}\n"
            "```\n"
        )

        review = code_help_tool(prompt, run_id)
        if not review:
            return "code_review unavailable (OPENROUTER_API_KEY not set or request failed). Proceed with caution or call final_answer."
        LOGGER.info("code_review run_id=%s review_len=%s", run_id, len(review))
        return review

    @tool("code_help")
    def code_help(main_rs: str, context: str, question: str) -> str:
        """
        Request help from an outside expert. Provide the current code
        and a question. For example, ask how to implement a pattern,
        how to fix an error, how to use a certain API, etc. The context
        can include build errors or other information the outside expert
        will need to help answer the question.
        """
        answer = code_help_helper(main_rs, context, question)
        return answer

    @tool("final_answer")
    def final_answer(main_rs: str) -> str:
        """
        Submit the final src/main.rs content when it is formatted and ready.
        """
        main_rs = normalize_rust_text(main_rs, field_name="main_rs")
        if not isinstance(main_rs, str) or not main_rs.strip():
            raise ValueError("main_rs must be a non-empty string")
        raise FinalAnswerException(main_rs)

    # TODO re-enable microsoft doc search
    return [rust_win_search, format_rust, evaluate_rust, code_review, code_help, final_answer], eval_state


def build_agent(tools):
    model_name = os.getenv("OLLAMA_MODEL", "glm-4.7-flash:latest")
    model_url = os.getenv("OLLAMA_BASE_URL", "http://127.0.0.1:11434")

    model = ChatOllama(
        model=model_name,
        base_url=model_url,
        temperature=0,
        think=False,
        num_predict=int(os.getenv("OLLAMA_NUM_PREDICT", "8000")),
    )
    context_limit = int(os.getenv("OLLAMA_CONTEXT_TOKENS", "6000"))

    system_prompt = """You solve Win32/Windows API programming problems in Rust.

Hard rules:
- Use ms_doc_search and rust_win_search to confirm any Win32 API signature/behavior and the correct Rust windows crate path/features.
- Use the code_help tool when you need help solving a problem in your code.
- Write stable Rust that compiles as src/main.rs. Do NOT include tests. The judge will append hidden tests after your code.
- You may call evaluate_rust to run build/clippy/tests. Keep iterating until it reports ok=true.
- When evaluate_rust returns, read the build diagnostics carefully. If errors say unresolved import or use of undeclared type, add the missing use statements at the top of the file and call evaluate_rust again with corrected code.
- Do NOT call evaluate_rust again with identical code after a failed build. You must make a concrete repair first.
- Only before the final output (once evaluate_rust passes without build errors), call format_rust on the final src/main.rs and use the formatted result.
- When you have a complete, formatted, tested solution, call the final_answer tool with the full src/main.rs content.
- Do not write a main() function.
- After evaluate_rust returns ok=true and before calling final_answer, call code_review
  with the current src/main.rs and the original problem text.
- If code_review returns VERDICT: NEEDS_CHANGES or REJECT, address all CRITICAL and MAJOR
  issues listed, then call evaluate_rust again to confirm the fix compiles and passes tests.
  Only call final_answer when code_review returns VERDICT: APPROVE or only MINOR issues remain.

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

The following helper may be used to create wide strings.
```
fn wide_null(s: &OsStr) -> Vec<u16> {
    // these imports are REQUIRED for the function to compile.
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
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

    agent = create_react_agent(
        model=model,
        tools=tools,
        prompt=system_prompt,
    )

    return agent.with_config({"recursion_limit": 40}), model, context_limit


def solve_problem(problem_text: str, unit_tests_private: str, max_attempts: int = 6) -> SolveResult:
    run_id = uuid.uuid4().hex[:8]
    eval_state: Dict[str, Any] = {}
    tools, _ = build_tools(unit_tests_private, eval_state)
    agent, agent_model, context_limit = build_agent(tools)
    tool_map = {t.name: t for t in tools}

    feedback = ""
    last_eval: Dict[str, Any] = {}
    refactor_done = False
    refactor_repair_rs: Optional[str] = None

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
        if refactor_repair_rs:
            run_input = (
                    problem_text
                    + "\n\n---\nREPAIR FEEDBACK:\n"
                    + truncate_feedback(feedback, 3000)
                    + "\n\n---\nREFACTORED CODE TO REPAIR (fix only the errors above, do not rewrite from scratch):\n```rust\n"
                    + refactor_repair_rs
                    + "\n```"
            )
            refactor_repair_rs = None
        elif feedback:
            run_input = problem_text + "\n\n---\nREPAIR FEEDBACK:\n" + truncate_feedback(feedback, 3000)
        else:
            run_input = problem_text

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
                "solve_problem final_answer_exception run_id=%s attempt=%s main_rs_len=%s",
                run_id,
                attempt,
                len(answer.main_rs),
            )
            main_rs = answer.main_rs
        else:
            duration_ms = int((time.perf_counter() - invoke_started) * 1000)
            msgs = result.get("messages") or []
            LOGGER.debug(
                "agent_invoke completed run_id=%s attempt=%s duration_ms=%s message_count=%s",
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
                summarize_tool_output("format_rust", formatted),
            )

        last_eval = eval_state.get("last", {})
        if not last_eval:
            tool_map["evaluate_rust"].invoke({"main_rs": main_rs})
            last_eval = eval_state.get("last", {})

        if last_eval.get("ok") is True:
            LOGGER.info(
                "solve_problem success run_id=%s attempt=%s summary=%s",
                run_id,
                attempt,
                preview_text(last_eval, limit=240),
            )
            if not refactor_done:
                main_rs = refactor_with_specialist(main_rs, problem_text, run_id)
                refactor_done = True
                eval_state.clear()
                tool_map["evaluate_rust"].invoke({"main_rs": main_rs})
                last_eval = eval_state.get("last", {})
                if last_eval.get("ok") is True:
                    refactor_repair_rs = None
                    return SolveResult(
                        main_rs=main_rs.rstrip() + "\n",
                        last_eval=last_eval,
                    )
                feedback = build_repair_message(last_eval, main_rs)
                refactor_repair_rs = main_rs
                continue
            return SolveResult(
                main_rs=main_rs.rstrip() + "\n",
                last_eval=last_eval,
            )

        feedback = build_repair_message(last_eval, main_rs)
        LOGGER.warning(
            "solve_problem attempt_failed run_id=%s attempt=%s feedback=%r",
            run_id,
            attempt,
            preview_text(feedback, limit=400),
        )

    LOGGER.error(
        "solve_problem exhausted_attempts run_id=%s max_attempts=%s last_eval=%r",
        run_id,
        max_attempts,
        preview_text(last_eval, limit=400),
    )
    raise RuntimeError(
        f"Failed to solve within {max_attempts} attempts. "
        f"Last eval:\n{summarize_eval(last_eval)}"
    )


if __name__ == "__main__":
    configure_logging()
    problem_text_input = open(
        "../dataset/simple-winapi/problems/0a5d7328-0ec4-4088-83d2-7e1c0e8b27c7.md",
        "r",
        encoding="utf-8",
    ).read()
    unit_tests_text_input = open(
        "../dataset/simple-winapi/tests/0a5d7328-0ec4-4088-83d2-7e1c0e8b27c7.rs",
        "r",
        encoding="utf-8",
    ).read()

    r = solve_problem(problem_text_input, unit_tests_text_input)
    print(r)
