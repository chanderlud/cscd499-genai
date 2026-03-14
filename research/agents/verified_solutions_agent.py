from dataclasses import dataclass
from langgraph.prebuilt import create_react_agent
from helpers import *


FIXED_DEPENDENCIES = open("../rust_dependencies.md", "r").read()


@dataclass
class SolveResult:
    main_rs: str
    last_eval: Dict[str, Any]


def build_agent(tools):
    model_name = os.getenv("OLLAMA_MODEL", "glm-4.7-flash:latest")
    model_url = os.getenv("OLLAMA_BASE_URL", "http://127.0.0.1:11434")

    model = ChatOllama(
        model=model_name,
        base_url=model_url,
        temperature=0,
        num_predict=int(os.getenv("OLLAMA_NUM_PREDICT", "8000")),
    )
    context_limit = int(os.getenv("OLLAMA_CONTEXT_TOKENS", "6000"))

    system_prompt = """You solve Win32/Windows API programming problems in Rust.

Hard rules:
- Use ms_doc_search and rust_win_search to confirm any Win32 API signature/behavior and the correct Rust windows crate path/features.
- Use the code_help tool when you need help solving a problem in your code.
- When calling code_help, always pass problem_text (the original problem statement) and doc_results (concatenated output from any ms_doc_search / rust_win_search calls already made for this problem).
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
    tools, _ = build_tools(unit_tests_private, FIXED_DEPENDENCIES, eval_state)
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
                salvaged_main_rs = extract_rust_from_messages(msgs)
                if salvaged_main_rs:
                    main_rs = salvaged_main_rs
                    LOGGER.warning(
                        "solve_problem salvaged_from_messages attempt=%s",
                        attempt + 1,
                    )
                elif eval_state.get("last", {}):
                    feedback = build_repair_message(
                        eval_state["last"], "", problem_text=problem_text
                    )
                    continue
                else:
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
                feedback = build_repair_message(last_eval, main_rs, problem_text=problem_text)
                refactor_repair_rs = main_rs
                continue
            return SolveResult(
                main_rs=main_rs.rstrip() + "\n",
                last_eval=last_eval,
            )

        feedback = build_repair_message(last_eval, main_rs, problem_text=problem_text)
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
        "../dataset/winapi-eval/problems/9beb547d-0701-4af0-a8c1-f7e09e0e6beb.md",
        "r",
        encoding="utf-8",
    ).read()
    unit_tests_text_input = open(
        "../dataset/winapi-eval/tests/9beb547d-0701-4af0-a8c1-f7e09e0e6beb.rs",
        "r",
        encoding="utf-8",
    ).read()

    r = solve_problem(problem_text_input, unit_tests_text_input)
    print(r)

    out = open("../dataset/winapi-eval/solutions/9beb547d-0701-4af0-a8c1-f7e09e0e6beb.md")
    out.write(r.main_rs)
