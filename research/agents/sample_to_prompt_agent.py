import argparse
import json
import os
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import List, Optional, Set

from helpers import LOGGER, configure_logging, env, openrouter_generate_code, preview_text


SYSTEM_PROMPT = """You are a technical problem-statement author for Rust/Win32 coding challenges.

Given only solution code, produce a concise, self-contained Markdown problem description that matches the style used in existing `winapi-eval-verified/problems/` files.

Required output structure:
- A short imperative title line (for example: `N) Title`)
- A `**Spec:**` bullet describing what the function must do
- A `**Constraints:**` bullet listing key requirements
- A `**Signature:**` bullet with a fenced Rust signature block
- An `**Example:**` bullet with a minimal usage snippet

Do not include any extra prose outside this structure.
"""

GENERATE_PROMPT_TEMPLATE = """Analyze the Rust source code below and reverse-engineer the coding challenge prompt it solves.

Requirements:
- Identify what the code does.
- Identify the primary Win32/Rust API being demonstrated.
- Write the problem statement as if this code is the reference solution.
- Output only the Markdown problem block, with no preamble or explanation.
- Begin your response with `PROBLEM:` on its own line.

## Solution code
```rust
{sample_code}
```
"""


@dataclass
class ProblemResult:
    problem_md: str
    sample_id: str


def _collect_input_samples(input_path: Path) -> List[Path]:
    if input_path.is_file() and input_path.suffix.lower() == ".rs":
        return [input_path]
    if input_path.is_dir():
        return sorted(path for path in input_path.rglob("*.rs") if path.is_file())
    raise ValueError(f"Input must be a .rs file or directory of .rs files: {input_path}")


def _extract_problem_md(response_text: str) -> Optional[str]:
    stripped = response_text.strip()
    if not stripped:
        return None

    lines = stripped.splitlines()
    sentinel_index = None
    for index, line in enumerate(lines):
        if line.strip() == "PROBLEM:":
            sentinel_index = index
            break

    if sentinel_index is None:
        problem_md = stripped
    else:
        problem_md = "\n".join(lines[sentinel_index + 1 :]).strip()

    return problem_md or None


def generate_problem_for_sample(sample_code: str, sample_path: Path) -> Optional[ProblemResult]:
    generate_prompt = GENERATE_PROMPT_TEMPLATE.format(sample_code=sample_code)
    messages = [
        {"role": "system", "content": SYSTEM_PROMPT},
        {"role": "user", "content": generate_prompt},
    ]

    response_text: Optional[str] = None
    for retry in range(2):
        response_text = openrouter_generate_code(messages)
        if response_text is not None:
            break
        if retry == 0:
            retry_prompt = (
                generate_prompt
                + "\n\nPlease begin with `PROBLEM:` on its own line and output only the Markdown problem block."
            )
            messages = [
                {"role": "system", "content": SYSTEM_PROMPT},
                {"role": "user", "content": retry_prompt},
            ]

    if response_text is None:
        LOGGER.warning("generate_problem_for_sample generation_failed sample=%s", sample_path)
        return None

    problem_md = _extract_problem_md(response_text)
    if not problem_md:
        LOGGER.warning(
            "generate_problem_for_sample empty_problem sample=%s response_preview=%s",
            sample_path,
            preview_text(response_text, limit=300),
        )
        return None

    sample_id = str(uuid.uuid4())
    LOGGER.info(
        "generate_problem_for_sample ok sample=%s id=%s preview=%s",
        sample_path,
        sample_id,
        preview_text(problem_md, limit=300),
    )
    return ProblemResult(problem_md=problem_md, sample_id=sample_id)


def _load_processed_sources(manifest_path: Path) -> Set[str]:
    processed: Set[str] = set()
    if not manifest_path.exists():
        return processed

    try:
        with manifest_path.open("r", encoding="utf-8") as handle:
            for line in handle:
                line = line.strip()
                if not line:
                    continue
                try:
                    record = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if record.get("ok") is not True:
                    continue
                source = record.get("source")
                if isinstance(source, str) and source.strip():
                    processed.add(source.strip())
    except OSError as exc:
        LOGGER.warning("process_input_folder manifest_read_failed path=%s error=%s", manifest_path, exc)

    return processed


def process_input_folder(input_dir: Path, output_dir: Path, overwrite: bool) -> None:
    input_dir = input_dir.expanduser().resolve()
    output_dir = output_dir.expanduser().resolve()
    problems_dir = output_dir / "problems"
    solutions_dir = output_dir / "solutions"
    manifest_path = output_dir / "manifest.jsonl"

    problems_dir.mkdir(parents=True, exist_ok=True)
    solutions_dir.mkdir(parents=True, exist_ok=True)

    sample_paths = _collect_input_samples(input_dir)
    processed_sources = _load_processed_sources(manifest_path)
    LOGGER.info(
        "process_input_folder start input=%s output=%s problems=%s solutions=%s manifest=%s samples=%s overwrite=%s cwd=%s model=%s",
        input_dir,
        output_dir,
        problems_dir,
        solutions_dir,
        manifest_path,
        len(sample_paths),
        overwrite,
        os.getcwd(),
        env("OPENROUTER_CODE_MODEL", env("OPENROUTER_REVIEW_MODEL", "openrouter/hunter-alpha")),
    )

    for sample_path in sample_paths:
        source_name = sample_path.name
        if source_name in processed_sources and not overwrite:
            LOGGER.info("process_input_folder skip source=%s reason=already_processed", source_name)
            continue

        try:
            sample_code = sample_path.read_text(encoding="utf-8")
        except OSError as exc:
            LOGGER.warning("process_input_folder read_failed sample=%s error=%s", sample_path, exc)
            continue

        result = generate_problem_for_sample(sample_code, sample_path)
        if result is None:
            LOGGER.warning("process_input_folder generation_failed sample=%s", sample_path)
            continue

        problem_path = problems_dir / f"{result.sample_id}.md"
        solution_path = solutions_dir / f"{result.sample_id}.rs"

        try:
            problem_path.write_text(result.problem_md, encoding="utf-8")
            solution_path.write_text(sample_code, encoding="utf-8")
            with manifest_path.open("a", encoding="utf-8") as handle:
                handle.write(
                    json.dumps(
                        {"id": result.sample_id, "source": source_name, "ok": True},
                        ensure_ascii=False,
                    )
                    + "\n"
                )
            processed_sources.add(source_name)
        except OSError as exc:
            LOGGER.warning(
                "process_input_folder write_failed sample=%s id=%s error=%s",
                sample_path,
                result.sample_id,
                exc,
            )
            continue

        LOGGER.info(
            "process_input_folder completed sample=%s id=%s problem=%s solution=%s",
            sample_path,
            result.sample_id,
            problem_path,
            solution_path,
        )


if __name__ == "__main__":
    configure_logging()
    parser = argparse.ArgumentParser(
        description="Sample-to-prompt agent: generate problem statements from Rust samples."
    )
    parser.add_argument(
        "--input-dir",
        "--input",
        dest="input_dir",
        required=True,
        help="Directory of .rs sample files.",
    )
    parser.add_argument(
        "--output-dir",
        "--output",
        dest="output_dir",
        required=True,
        help="Output root; problems/ and solutions/ are created here.",
    )
    parser.add_argument(
        "--overwrite",
        action="store_true",
        help="Re-generate even if already processed.",
    )
    args = parser.parse_args()

    process_input_folder(Path(args.input_dir), Path(args.output_dir), args.overwrite)
