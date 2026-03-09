#!/usr/bin/env python3
import argparse
import json
import os
import re
import sys
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, List, Optional

import httpx

from langchain.agents import create_agent
from langchain_core.messages import ToolMessage
from langchain_core.tools import tool
from langchain_openrouter import ChatOpenRouter

from verified_solutions_agent import (
    FIXED_DEPENDENCIES,
    _apply_context_window,
    _compress_old_tool_messages,
    _env,
    _extract_json_object,
    _summarize_eval,
    _summarize_step,
    _truncate_feedback,
)

STUB_IMPORTS = """use std::{slice::Iter, cmp::{max, self}, mem::replace, collections::{HashSet, HashMap}, ops::Index, ascii::AsciiExt};
use rand::Rng;
use regex::Regex;
use md5;
use std::any::{Any, TypeId};"""


class FinalAnswerException(Exception):
    def __init__(self, unit_tests_rs: str):
        super().__init__("final_answer")
        self.unit_tests_rs = unit_tests_rs


class GiveUpException(Exception):
    pass


@dataclass
class VerifyResult:
    unit_tests_rs: str
    last_eval: Dict[str, Any]
    gave_up: bool = False
    give_up_reason: str = ""


def _extract_rust_blocks(problem_md: str) -> List[str]:
    return re.findall(r"```(?:rust)?\s*(.*?)```", problem_md, flags=re.DOTALL | re.IGNORECASE)


def _extract_function_signature(problem_md: str) -> str:
    candidates = _extract_rust_blocks(problem_md)
    candidates.append(problem_md)

    signature_pattern = re.compile(
        r"((?:pub\s+)?(?:unsafe\s+)?(?:async\s+)?(?:extern\s+\"[^\"]+\"\s+)?fn\s+\w+\s*\([^)]*\)"
        r"(?:\s*->\s*[^\{;\n]+)?(?:\s+where\s+[^\{;]+)?)",
        flags=re.DOTALL,
    )

    for text in candidates:
        normalized = re.sub(r"\s+", " ", text)
        match = signature_pattern.search(normalized)
        if match:
            return match.group(1).strip().rstrip(";")

    raise ValueError("Could not find a Rust function signature in the problem markdown")


def generate_stub(problem_md: str) -> str:
    signature = _extract_function_signature(problem_md)
    return (
        f"{STUB_IMPORTS}\n\n"
        f"{signature} {{\n"
        f"    unimplemented!()\n"
        f"}}\n\n"
        f"fn main() {{}}\n"
    )


def _build_eval_summary(resp: Dict[str, Any]) -> Dict[str, Any]:
    build = resp.get("build", {}) if isinstance(resp.get("build"), dict) else {}
    summary = dict(resp)
    summary["ok"] = build.get("ok") is True
    summary["success_criterion"] = "build.ok"
    return summary


def _extract_message_text(message: Any) -> str:
    content = getattr(message, "content", "")
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        parts = []
        for part in content:
            if isinstance(part, dict):
                text = part.get("text")
                if isinstance(text, str):
                    parts.append(text)
        return "\n".join(parts)
    return ""


def _is_tool_request(obj: Dict[str, Any]) -> bool:
    return (
        isinstance(obj, dict)
        and isinstance(obj.get("name"), str)
        and isinstance(obj.get("arguments"), dict)
    )


def _is_final_answer(obj: Dict[str, Any]) -> bool:
    return isinstance(obj, dict) and "unit_tests_rs" in obj and isinstance(obj["unit_tests_rs"], str)


def build_tools(problem_md: str, current_tests_holder: List[str]) -> Dict[str, Any]:
    msdocs_base = _env("MSDOCS_BASE_URL", "http://127.0.0.1:3000")
    rustdocs_base = _env("RUSTDOCS_BASE_URL", "http://127.0.0.1:3001")
    eval_base = _env("RUST_EVAL_BASE_URL", "http://127.0.0.1:3002")

    client = httpx.Client(timeout=30.0)
    stub_holder = [generate_stub(problem_md)]

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
        Search Microsoft docs (Win32) for C/C++ signatures, behavior, examples, etc.
        """
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
            return json.dumps(data, ensure_ascii=False, indent=2)[:18000]
        except Exception as e:
            return f"ms_doc_search error: {type(e).__name__}: {e}"

    @tool("rust_win_search")
    def rust_win_search(q: str, kind: Optional[str] = "function", limit: int = 5) -> str:
        """
        Search Rust Windows API docs for windows crate paths, signatures, etc.
        """
        try:
            r = client.get(
                f"{rustdocs_base}/search",
                params={"q": q, "kind": kind, "limit": limit},
            )
            r.raise_for_status()
            data = r.json()
            return json.dumps(data, ensure_ascii=False, indent=2)[:18000]
        except Exception as e:
            return f"rust_win_search error: {type(e).__name__}: {e}"

    @tool("format_rust")
    def format_rust(snippet: str) -> str:
        """
        Format Rust snippet using the formatting API. Returns formatted code (or error text).
        """
        try:
            r = client.post(f"{eval_base}/format", json={"snippet": snippet})
            r.raise_for_status()
            data = r.json()
            if data.get("ok") and data.get("formatted"):
                return data["formatted"]
            return f"format_rust failed: {json.dumps(data, ensure_ascii=False)[:12000]}"
        except Exception as e:
            return f"format_rust error: {type(e).__name__}: {e}"

    @tool("generate_stub")
    def generate_stub_tool(stub_main_rs: str) -> str:
        """
        Submit a compilable src/main.rs stub for the function under test. The stub must compile
        (use unimplemented!() for the body). Returns "ok" or a compile error to help you fix the
        stub before testing.
        """
        try:
            r = client.post(
                f"{eval_base}/evaluate",
                json={
                    "main_rs": stub_main_rs.rstrip() + "\n",
                    "dependencies": FIXED_DEPENDENCIES,
                },
            )
            r.raise_for_status()
            data = r.json()
            build = data.get("build", {}) if isinstance(data.get("build"), dict) else {}
            if build.get("ok") is True:
                stub_holder[0] = stub_main_rs.rstrip() + "\n"
                return "ok"
            return _summarize_step(build or {"name": "build", "ok": False})
        except Exception as e:
            return f"generate_stub error: {type(e).__name__}: {e}"

    @tool("evaluate_tests")
    def evaluate_tests(unit_tests_rs: str) -> str:
        """
        Evaluate the given #[cfg(test)] test block against the stub. Runs build + cargo test.
        Returns the EvaluateResponse summary. Success criterion: build.ok=True only. Tests do
        not need to pass; the goal is to verify the test file compiles correctly. The stub
        implementation will panic on execution, but that is expected during verification.
        """
        try:
            current_tests_holder[0] = unit_tests_rs.rstrip() + "\n"
            combined = stub_holder[0].rstrip() + "\n\n" + current_tests_holder[0]
            r = client.post(
                f"{eval_base}/evaluate",
                json={"main_rs": combined, "dependencies": FIXED_DEPENDENCIES},
            )
            r.raise_for_status()
            data = r.json()
            return json.dumps(_build_eval_summary(data), ensure_ascii=False, indent=2)[:24000]
        except Exception as e:
            return f"evaluate_tests error: {type(e).__name__}: {e}"

    @tool("give_up")
    def give_up(reason: str) -> str:
        """
        Use this if the problem fundamentally cannot be unit-tested automatically (e.g., requires
        interactive UI, real hardware device, persistent system state, or untestable constraints).
        Calling this tool ends the verification process.
        """
        raise GiveUpException(reason)

    @tool("final_answer")
    def final_answer(unit_tests_rs: str) -> str:
        """
        Submit the final repaired Rust test module when it is ready.
        """
        if not isinstance(unit_tests_rs, str) or not unit_tests_rs.strip():
            raise ValueError("unit_tests_rs must be a non-empty string")
        raise FinalAnswerException(unit_tests_rs)

    return {
        "ms_doc_search": ms_doc_search,
        "rust_win_search": rust_win_search,
        "format_rust": format_rust,
        "generate_stub": generate_stub_tool,
        "evaluate_tests": evaluate_tests,
        "give_up": give_up,
        "final_answer": final_answer,
    }


def build_agent(tools: Dict[str, Any]) -> Any:
    model = ChatOpenRouter(
        model=os.getenv("OPENROUTER_MODEL", "arcee-ai/trinity-large-preview:free"),
        temperature=0,
        max_tokens=16000,
        max_retries=2,
    )
    context_limit = int(os.getenv("OLLAMA_CONTEXT_TOKENS", "6000"))

    system_prompt = """You are an expert Rust test engineer for Windows API problems.

Goal:
- Repair and improve the provided Rust test file so it compiles successfully.
- Success is build.ok=True only. Ignore whether the tests pass at runtime because the stub
  implementation may panic.

Hard rules:
- unit_tests_rs must be a full #[cfg(all(test, windows))] mod tests { ... } block.
- Include necessary imports inside the test module, including use super::*; when needed.
- You may add helper functions inside the mod tests block.
- You may remove or rewrite incorrect tests if they are irreparably wrong.
- Use ms_doc_search and rust_win_search to identify correct Windows imports and symbols when needed.
- You must submit a compilable stub with generate_stub before evaluating tests.
- You must call evaluate_tests after every repair attempt.
- When you have a complete, compilable test module, call the final_answer tool with the full unit_tests_rs content.
- You must call give_up if the problem cannot be unit-tested automatically because it depends on
  interactive UI, hardware devices, or irreducibly stateful system behavior.

Quality target:
- Match the quality bar of verified examples: proper cfg guard, correct imports, deterministic tests,
  useful helpers, and edge-case coverage where feasible.
"""

    agent = create_agent(
        model=model,
        tools=list(tools.values()),
        system_prompt=system_prompt,
    )

    return agent.with_config({"recursion_limit": 15}), model, context_limit


def _build_run_input(problem_text: str, unit_tests_rs: str, feedback: str) -> str:
    parts = [
        "Repair this Rust Windows test module so it compiles against a stub implementation.",
        "",
        "Problem markdown:",
        problem_text.strip(),
        "",
        "Current candidate tests:",
        unit_tests_rs.strip(),
    ]
    if feedback:
        parts.extend(["", "---", "REPAIR FEEDBACK:", _truncate_feedback(feedback, 3000).strip()])
    return "\n".join(parts).strip() + "\n"


def verify_tests(problem_text: str, unit_tests_rs: str, max_attempts: int = 5) -> VerifyResult:
    current_tests_holder = [unit_tests_rs.rstrip() + "\n"]
    tools = build_tools(problem_text, current_tests_holder)
    agent, agent_model, context_limit = build_agent(tools)

    feedback = ""
    last_eval: Dict[str, Any] = {}

    for attempt in range(1, max_attempts + 1):
        run_input = _build_run_input(problem_text, current_tests_holder[0], feedback)
        final_tests: Optional[str] = None
        try:
            messages = [{"role": "user", "content": run_input}]
            for tool_hops in range(40):
                messages = _compress_old_tool_messages(messages, keep_last_n=1)
                messages = _apply_context_window(messages, model=agent_model, max_tokens=context_limit)
                try:
                    result = agent.invoke({"messages": messages})
                except FinalAnswerException as answer:
                    final_tests = answer.unit_tests_rs
                    break

                msgs = result.get("messages") or []
                last = msgs[-1] if msgs else None
                content = _extract_message_text(last)
                obj = _extract_json_object(content)

                if _is_tool_request(obj):
                    tool_name = obj["name"]
                    tool_args = obj["arguments"]

                    if tool_name not in tools:
                        raise RuntimeError(f"Model requested unknown tool: {tool_name}")

                    try:
                        tool_out = tools[tool_name].invoke(tool_args)
                    except FinalAnswerException as answer:
                        final_tests = answer.unit_tests_rs
                        break

                    messages = list(msgs) + [
                        ToolMessage(
                            content=tool_out,
                            tool_call_id=f"manual_{tool_hops}",
                            name=tool_name,
                        )
                    ]
                    continue

                if _is_final_answer(obj):
                    final_tests = obj["unit_tests_rs"]
                    break

                raise RuntimeError("Agent finished without calling final_answer.")
            else:
                raise RuntimeError("Too many tool hops without producing a final answer.")

            if final_tests is None:
                raise RuntimeError("Agent finished without calling final_answer.")
        except FinalAnswerException as answer:
            final_tests = answer.unit_tests_rs
        except GiveUpException as e:
            return VerifyResult(
                unit_tests_rs=current_tests_holder[0],
                last_eval={"ok": False, "attempts": attempt},
                gave_up=True,
                give_up_reason=str(e),
            )

        try:
            eval_json = tools["evaluate_tests"].invoke({"unit_tests_rs": final_tests})
            try:
                last_eval = json.loads(eval_json)
            except Exception:
                last_eval = {"ok": False, "raw": eval_json}

            last_eval["attempts"] = attempt
            if last_eval.get("ok") is True:
                return VerifyResult(
                    unit_tests_rs=final_tests.rstrip() + "\n",
                    last_eval=last_eval,
                )

            feedback = _summarize_eval(last_eval)
        except GiveUpException as e:
            return VerifyResult(
                unit_tests_rs=current_tests_holder[0],
                last_eval={"ok": False, "attempts": attempt},
                gave_up=True,
                give_up_reason=str(e),
            )

    raise RuntimeError(
        f"Failed to verify within {max_attempts} attempts. "
        f"Last eval:\n{_summarize_eval(last_eval)}"
    )


def is_uuid(stem: str) -> bool:
    try:
        uuid.UUID(stem)
        return True
    except ValueError:
        return False


def index_dir(dir_path: Path, ext: str, validate_uuid: bool = True) -> Dict[str, Path]:
    if not dir_path.exists() or not dir_path.is_dir():
        raise FileNotFoundError(f"Not a directory: {dir_path}")

    out: Dict[str, Path] = {}
    for path in dir_path.iterdir():
        if not path.is_file():
            continue
        if path.suffix.lower() != ext.lower():
            continue

        stem = path.stem
        if validate_uuid and not is_uuid(stem):
            continue
        if stem in out:
            raise ValueError(f"Duplicate {ext} filename stem '{stem}' in {dir_path}")
        out[stem] = path

    return out


def main() -> int:
    root = Path(__file__).resolve().parent
    default_problems = root / "dataset" / "staging" / "problems"
    default_tests = root / "dataset" / "staging" / "tests"
    default_out = root / "dataset" / "staging" / "verified-tests"

    parser = argparse.ArgumentParser(
        description="Repair and verify Rust unit tests for staged Windows API problems."
    )
    parser.add_argument("--problems-dir", type=Path, default=default_problems)
    parser.add_argument("--tests-dir", type=Path, default=default_tests)
    parser.add_argument("--out-tests-dir", type=Path, default=default_out)
    parser.add_argument("--overwrite", action="store_true")
    args = parser.parse_args()

    problems = index_dir(args.problems_dir, ".md", validate_uuid=True)
    tests = index_dir(args.tests_dir, ".rs", validate_uuid=True)
    common_ids = sorted(set(problems) & set(tests))

    args.out_tests_dir.mkdir(parents=True, exist_ok=True)

    summary: Dict[str, Dict[str, Any]] = {}
    ok_count = 0
    gave_up_count = 0
    failed_count = 0
    skipped_count = 0

    for problem_id in common_ids:
        out_path = args.out_tests_dir / f"{problem_id}.rs"
        if out_path.exists() and not args.overwrite:
            summary[problem_id] = {
                "ok": False,
                "gave_up": False,
                "reason": "output exists; use --overwrite to replace",
                "attempts": 0,
            }
            skipped_count += 1
            print(f"[skip] {problem_id}: {out_path} exists", file=sys.stderr)
            continue

        try:
            problem_text = problems[problem_id].read_text(encoding="utf-8-sig")
            unit_tests_text = tests[problem_id].read_text(encoding="utf-8-sig")
            result = verify_tests(problem_text, unit_tests_text)

            summary[problem_id] = {
                "ok": not result.gave_up and result.last_eval.get("ok") is True,
                "gave_up": result.gave_up,
                "reason": result.give_up_reason,
                "attempts": result.last_eval.get("attempts", 0),
            }

            if result.gave_up:
                gave_up_count += 1
                print(f"[give_up] {problem_id}: {result.give_up_reason}", file=sys.stderr)
            else:
                out_path.write_text(result.unit_tests_rs, encoding="utf-8")
                ok_count += 1
                print(f"[ok] {problem_id}", file=sys.stderr)
        except Exception as e:
            failed_count += 1
            summary[problem_id] = {
                "ok": False,
                "gave_up": False,
                "reason": f"{type(e).__name__}: {e}",
                "attempts": 0,
            }
            print(f"[fail] {problem_id}: {type(e).__name__}: {e}", file=sys.stderr)

    results_path = args.out_tests_dir / "verify_results.json"
    results_path.write_text(
        json.dumps(summary, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    print(
        (
            f"Verified {len(common_ids)} pair(s): ok={ok_count} gave_up={gave_up_count} "
            f"failed={failed_count} skipped={skipped_count}. "
            f"Summary: {results_path}"
        ),
        file=sys.stderr,
    )
    return 0 if failed_count == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
