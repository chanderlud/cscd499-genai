import json
import os
import re
from dataclasses import dataclass
from typing import Any, Dict, Optional

import httpx
from pydantic import BaseModel, Field

from langchain_openrouter import ChatOpenRouter
from langchain.agents import AgentExecutor, create_tool_calling_agent
from langchain_core.prompts import ChatPromptTemplate, MessagesPlaceholder
from langchain_core.tools import tool

FIXED_DEPENDENCIES = """
regex = "*"
rand = "*"
md5 = "*"
windows = { version = "0.62.2", features = [
    "Win32_System_Com", 
    "Win32_UI", 
    "Win32_UI_Shell",
    "Win32_System_Ole", 
    "Win32_System_WindowsProgramming", 
    "Win32_System_SystemInformation", 
    "Win32_Storage", 
    "Win32_Storage_FileSystem",
    "Win32_Security", 
    "Win32_System_Pipes",
    "Win32_System_Threading", 
    "Win32_System_IO"] }
""".strip() + "\n"


class FinalAnswer(BaseModel):
    main_rs: str = Field(
        ...,
        description="Full Rust source for src/main.rs. Must compile. Do NOT include tests.",
    )


def _env(name: str, default: str) -> str:
    return os.getenv(name, default).rstrip("/")


def _extract_json_object(text: str) -> Dict[str, Any]:
    """
    Best-effort extraction of a JSON object from LLM output.
    Enforces: must be a single object with {main_rs}.
    (If the model still outputs 'dependencies', it is ignored.)
    """
    text = text.strip()

    # If it's already pure JSON
    if text.startswith("{") and text.endswith("}"):
        obj = json.loads(text)
        if "main_rs" not in obj:
            raise ValueError("JSON must contain key: main_rs")
        return obj

    # Pull first {...} blob (handles extra chatter)
    m = re.search(r"\{[\s\S]*\}", text)
    if not m:
        raise ValueError("No JSON object found in model output.")
    obj = json.loads(m.group(0))
    if "main_rs" not in obj:
        raise ValueError("JSON must contain key: main_rs")
    return obj


def _ensure_empty_main(main_rs: str) -> str:
    """
    The evaluator requires an empty main() present in the snippet.
    If user code already has main(), we leave it.
    Otherwise append fn main() {}.
    """
    if re.search(r"\bfn\s+main\s*\(", main_rs):
        return main_rs
    return main_rs.rstrip() + "\n\nfn main() {}\n"


def _summarize_step(report: Dict[str, Any]) -> str:
    # Compact summary for the LLM to fix issues without drowning it.
    diag = report.get("diagnostics", {}) or {}
    errs = diag.get("errors", 0)
    warns = diag.get("warnings", 0)

    stdout = report.get("stdout", "") or ""
    stderr = report.get("stderr", "") or ""

    # Keep the tail (most relevant)
    def tail(s: str, n: int = 4000) -> str:
        s = s.strip()
        return s[-n:] if len(s) > n else s

    return (
        f"{report.get('name','step')} ok={report.get('ok')} "
        f"timed_out={report.get('timed_out')} exit_code={report.get('exit_code')} "
        f"errors={errs} warnings={warns}\n"
        f"--- stderr (tail) ---\n{tail(stderr)}\n"
        f"--- stdout (tail) ---\n{tail(stdout)}\n"
    )


def _summarize_eval(resp: Dict[str, Any]) -> str:
    parts = [f"GLOBAL ok={resp.get('ok')} project_id={resp.get('project_id')}"]
    for k in ["build", "clippy", "tests"]:
        if k in resp and isinstance(resp[k], dict):
            parts.append(_summarize_step(resp[k]))
    return "\n\n".join(parts)


def build_tools(unit_tests_private: str):
    msdocs_base = _env("MSDOCS_BASE_URL", "http://127.0.0.1:3000")
    rustdocs_base = _env("RUSTDOCS_BASE_URL", "http://127.0.0.1:3001")
    eval_base = _env("RUST_EVAL_BASE_URL", "http://127.0.0.1:3002")

    client = httpx.Client(timeout=30.0)

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
            # Return compact-ish JSON (top results only if present)
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

    @tool("evaluate_rust")
    def evaluate_rust(main_rs: str, dependencies: str) -> str:
        """
        Build + clippy + run hidden tests. The harness appends hidden tests privately.
        Returns the EvaluateResponse JSON as a string.
        """
        try:
            full_main = (
                    _ensure_empty_main(main_rs).rstrip()
                    + "\n\n"
                    + unit_tests_private.strip()
                    + "\n"
            )
            r = client.post(
                f"{eval_base}/evaluate",
                json={"main_rs": full_main, "dependencies": dependencies},
            )
            r.raise_for_status()
            data = r.json()
            return json.dumps(data, ensure_ascii=False, indent=2)[:24000]
        except Exception as e:
            return f"evaluate_rust error: {type(e).__name__}: {e}"

    return [ms_doc_search, rust_win_search, format_rust, evaluate_rust]


def build_agent(tools):
    # OpenRouter model (tool calling supported by ChatOpenRouter)
    headers = {}
    if os.getenv("OPENROUTER_HTTP_REFERER"):
        headers["HTTP-Referer"] = os.getenv("OPENROUTER_HTTP_REFERER")
    if os.getenv("OPENROUTER_TITLE"):
        headers["X-OpenRouter-Title"] = os.getenv("OPENROUTER_TITLE")

    llm = ChatOpenRouter(
        model=os.getenv("OPENROUTER_MODEL", "openai/gpt-4.1"),
        temperature=0,
        max_tokens=4000,
        max_retries=2,
        default_headers=headers or None,
    )

    prompt = ChatPromptTemplate.from_messages(
        [
            (
                "system",
                """You solve Win32/Windows API programming problems in Rust.

Hard rules:
- Use ms_doc_search and rust_win_search to confirm any Win32 API signature/behavior and the correct Rust windows crate path/features.
- Write stable Rust that compiles as src/main.rs. Do NOT include tests. The judge will append hidden tests after your code.
- You may call evaluate_rust to run build/clippy/tests. Keep iterating until it reports ok=true.
- Before final output, call format_rust on the final src/main.rs and use the formatted result.
- Final response MUST be a single JSON object with EXACT key: main_rs. No extra text.

Quality rules:
- Prefer safe wrappers. If unsafe is required, minimize scope and justify via comments.
- Handle Win32 error returns properly (GetLastError / HRESULT / WSAGetLastError as appropriate).
- Dependencies are fixed and MUST NOT be output by the model.""",
            ),
            ("human", "{input}"),
            MessagesPlaceholder(variable_name="agent_scratchpad"),
        ]
    )

    agent = create_tool_calling_agent(llm, tools, prompt)
    executor = AgentExecutor(agent=agent, tools=tools, verbose=True, max_iterations=20)
    return executor


@dataclass
class SolveResult:
    main_rs: str
    last_eval: Dict[str, Any]


def solve_problem(problem_text: str, unit_tests_private: str, max_attempts: int = 6) -> SolveResult:
    tools = build_tools(unit_tests_private)
    executor = build_agent(tools)

    # Keep a running "feedback" blob. This is what actually drives repairs.
    feedback = ""
    last_eval: Dict[str, Any] = {}

    for attempt in range(1, max_attempts + 1):
        run_input = problem_text if not feedback else (problem_text + "\n\n---\nREPAIR FEEDBACK:\n" + feedback)
        out = executor.invoke({"input": run_input})
        raw = out.get("output", "")

        # Parse the agent's claimed final answer.
        obj = _extract_json_object(raw)
        answer = FinalAnswer(**obj)

        formatted = tools[2].invoke({"snippet": answer.main_rs})  # format_rust
        if isinstance(formatted, str) and not formatted.startswith("format_rust error"):
            answer.main_rs = formatted

        eval_json = tools[3].invoke({"main_rs": answer.main_rs, "dependencies": FIXED_DEPENDENCIES})  # evaluate_rust
        try:
            last_eval = json.loads(eval_json)
        except Exception:
            last_eval = {"ok": False, "raw": eval_json}

        if last_eval.get("ok") is True:
            return SolveResult(
                main_rs=answer.main_rs.rstrip() + "\n",
                last_eval=last_eval,
            )

        feedback = _summarize_eval(last_eval)

    raise RuntimeError(
        f"Failed to solve within {max_attempts} attempts. Last eval:\n{_summarize_eval(last_eval)}"
    )