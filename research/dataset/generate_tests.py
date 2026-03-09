#!/usr/bin/env python3
"""
Generate Rust unit tests from .md problem statements using OpenRouter.

- Reads *.md files from an input folder
- Inserts each into a prompt template that requests unit tests
- Calls OpenRouter Chat Completions API
- Extracts the Rust code block (```rust ... ```) from the model response
- Writes tests to output folder as <input_stem>.rs

Environment:
  OPENROUTER_API_KEY   (required)
  OPENROUTER_MODEL     (optional default: "openai/gpt-4.1-mini")
  OPENROUTER_BASE_URL  (optional default: "https://openrouter.ai/api/v1")
  OPENROUTER_REFERER   (optional, for OpenRouter attribution/rankings)
  OPENROUTER_TITLE     (optional, for OpenRouter attribution/rankings)

Example:
  export OPENROUTER_API_KEY="..."
  python generate_tests.py ./problems_md ./tests --model "anthropic/claude-3.7-sonnet" --overwrite
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Optional, Tuple

import requests


DEFAULT_MODEL = os.getenv("OPENROUTER_MODEL", "hf.co/Fortytwo-Network/Strand-Rust-Coder-14B-v1-GGUF:Q8_0")
DEFAULT_BASE_URL = os.getenv("OPENROUTER_BASE_URL", "http://localhost:11434/v1")


PROMPT_TEMPLATE = """You are an expert Rust developer and test engineer.

TASK:
Write a Rust test file for the programming problem described below.
Simply verify the behavior of the function while attempting to catch edge cases.

OUTPUT RULES (IMPORTANT):
- Output ONLY one Rust fenced code block (```rust ... ```). No prose, no extra blocks.
- Prefer deterministic tests with clear expected values.
- Do not implement the programming problem, assume that the function described by the problem will be imported from super.
- Do not assume that functions other than the function described in the problem will be available
- You may use the windows crate, but other external dependencies should be avoided
- Do not implement mock windows API related functions or types, use the windows crate

Always write tests within a block like this, do not write code outside this block:
#[cfg(test)]
mod tests {{
    use super::*;
}}

PROBLEM (markdown):
{problem_md}
"""


RUST_FENCE_RE = re.compile(
    r"```(?:rust|rs)\s*(?P<code>[\s\S]*?)\s*```",
    re.IGNORECASE,
)

ANY_FENCE_RE = re.compile(
    r"```\s*(?:[a-zA-Z0-9_+-]+)?\s*(?P<code>[\s\S]*?)\s*```",
    re.IGNORECASE,
)


@dataclass
class OpenRouterConfig:
    api_key: str
    model: str
    base_url: str
    referer: Optional[str] = None
    title: Optional[str] = None
    timeout_s: int = 60
    max_tokens: int = 1800
    temperature: float = 0.2
    retries: int = 6


def extract_rust_code_block(text: str) -> Tuple[str, str]:
    """
    Returns (code, mode) where mode is:
      - "rust" if ```rust/rs``` block found
      - "any" if any fenced code block found
      - "raw" if no fenced block found (returns whole text)
    """
    m = RUST_FENCE_RE.search(text)
    if m:
        return m.group("code").strip() + "\n", "rust"

    m = ANY_FENCE_RE.search(text)
    if m:
        return m.group("code").strip() + "\n", "any"

    return text.strip() + "\n", "raw"


def build_prompt(problem_md: str, prefix: str, solution_path_template: str) -> str:
    solution_path = solution_path_template.format(prefix=prefix)
    return PROMPT_TEMPLATE.format(prefix=prefix, solution_path=solution_path, problem_md=problem_md.strip())


def call_openrouter(cfg: OpenRouterConfig, prompt: str) -> str:
    url = f"{cfg.base_url.rstrip('/')}/chat/completions"

    headers = {
        "Authorization": f"Bearer {cfg.api_key}",
        "Content-Type": "application/json",
    }
    # Optional attribution headers (OpenRouter accepts these; harmless if omitted)
    if cfg.referer:
        headers["HTTP-Referer"] = cfg.referer
    if cfg.title:
        headers["X-Title"] = cfg.title  # X-OpenRouter-Title is also accepted; X-Title is widely used.

    payload = {
        "model": cfg.model,
        "messages": [{"role": "user", "content": prompt}],
        "temperature": cfg.temperature,
        "max_tokens": cfg.max_tokens,
    }

    last_err = None
    for attempt in range(cfg.retries + 1):
        try:
            resp = requests.post(url, headers=headers, data=json.dumps(payload), timeout=cfg.timeout_s)
            if resp.status_code == 200:
                data = resp.json()
                # OpenAI-compatible shape: choices[0].message.content
                return data["choices"][0]["message"]["content"]

            # Retry on rate limiting and typical transient failures
            if resp.status_code in (429, 500, 502, 503, 504):
                wait_s = min(2 ** attempt, 30) + (0.1 * attempt)
                time.sleep(wait_s)
                continue

            # Non-retryable HTTP errors
            raise RuntimeError(f"HTTP {resp.status_code}: {resp.text[:500]}")

        except (requests.Timeout, requests.ConnectionError) as e:
            last_err = e
            wait_s = min(2 ** attempt, 30) + (0.1 * attempt)
            time.sleep(wait_s)
            continue

    raise RuntimeError(f"OpenRouter request failed after retries. Last error: {last_err!r}")


def main() -> int:
    ap = argparse.ArgumentParser(description="Generate Rust unit tests from .md problems via OpenRouter.")
    ap.add_argument("input_dir", type=Path, help="Folder containing .md problem files")
    ap.add_argument("output_dir", type=Path, help="Folder to write .rs test files into (created if missing)")
    ap.add_argument("--model", default=DEFAULT_MODEL, help=f"OpenRouter model id (default: {DEFAULT_MODEL})")
    ap.add_argument("--base-url", default=DEFAULT_BASE_URL, help=f"OpenRouter base URL (default: {DEFAULT_BASE_URL})")
    ap.add_argument("--solution-path-template", default="../src/bin/{prefix}.rs",
                    help='Template path used inside tests via #[path="..."]. Use {prefix}. '
                         'Default: ../src/bin/{prefix}.rs')
    ap.add_argument("--overwrite", action="store_true", help="Overwrite existing .rs files")
    ap.add_argument("--dry-run", action="store_true", help="Do not call API; just show which files would be processed")
    ap.add_argument("--timeout", type=int, default=60, help="HTTP timeout seconds (default: 60)")
    ap.add_argument("--max-tokens", type=int, default=1800, help="max_tokens for generation (default: 1800)")
    ap.add_argument("--temperature", type=float, default=0.2, help="temperature for generation (default: 0.2)")
    ap.add_argument("--retries", type=int, default=6, help="Retry count for transient errors (default: 6)")
    args = ap.parse_args()

    api_key = os.getenv("OPENROUTER_API_KEY", "ollama")
    if not api_key and not args.dry_run:
        print("ERROR: OPENROUTER_API_KEY is not set.", file=sys.stderr)
        return 2

    cfg = OpenRouterConfig(
        api_key=api_key or "",
        model=args.model,
        base_url=args.base_url,
        referer=os.getenv("OPENROUTER_REFERER"),
        title=os.getenv("OPENROUTER_TITLE"),
        timeout_s=args.timeout,
        max_tokens=args.max_tokens,
        temperature=args.temperature,
        retries=args.retries,
    )

    in_dir = args.input_dir
    out_dir = args.output_dir
    out_dir.mkdir(parents=True, exist_ok=True)

    md_files = sorted(in_dir.glob("*.md"))
    if not md_files:
        print(f"No .md files found in: {in_dir}", file=sys.stderr)
        return 1

    for md_path in md_files:
        prefix = md_path.stem
        out_path = out_dir / f"{prefix}.rs"

        if out_path.exists() and not args.overwrite:
            print(f"SKIP (exists): {out_path}")
            continue

        problem_md = md_path.read_text(encoding="utf-8", errors="replace")
        prompt = build_prompt(problem_md=problem_md, prefix=prefix, solution_path_template=args.solution_path_template)

        print(f"PROCESS: {md_path.name} -> {out_path.name}")

        if args.dry_run:
            continue

        try:
            response_text = call_openrouter(cfg, prompt)
            code, mode = extract_rust_code_block(response_text)

            header = (
                f"// Auto-generated tests for: {md_path.name}\n"
                f"// Model: {cfg.model}\n"
                f"// Extraction: {mode}\n\n"
            )

            out_path.write_text(header + code, encoding="utf-8")
        except Exception as e:
            print(f"ERROR processing {md_path.name}: {e}", file=sys.stderr)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())