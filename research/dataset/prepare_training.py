#!/usr/bin/env python
"""
Prepare a Rust problem/solution dataset for Qwen SFT.

Input layout:
    <root>/problems/<uuid>.md
    <root>/solutions/<uuid>.rs

Features:
- Pairs problems and solutions by matching filename stem (UUID or any shared basename).
- Detects duplicate problems via normalized markdown text.
- Emits warnings and filters duplicate problems from the training dataset.
- Computes per-file stats:
    * problem tokens
    * code tokens
    * total / non-empty / code LOC
    * windows crate imports per file
- Produces summary artifacts:
    * dataset JSONL
    * summary JSON
    * per-file stats CSV
    * duplicate report JSON
    * windows import histogram CSV

Output dataset format:
   {
     "id": "...",
     "prompt": [{"role": "system", "content": "..."}, {"role": "user", "content": "..."}],
     "completion": [{"role": "assistant", "content": "..."}],
     "metadata": {...}
   }

Example:
    python prepare_training.py ^
      --root data ^
      --secondary-roots extra_data1 ^
      --secondary-roots extra_data2 ^
      --output-dir prepared ^
      --model-name Qwen/Qwen2.5-Coder-7B-Instruct ^
      --system-prompt "You are a precise Rust coding assistant. Return correct, idiomatic Rust."
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import math
import random
import re
import statistics
import sys
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, Iterable, List, Sequence, Tuple

from transformers import AutoTokenizer


@dataclass
class ExampleRecord:
    sample_id: str
    problem_path: Path
    solution_path: Path
    problem_text: str
    solution_text: str


@dataclass
class FileStats:
    sample_id: str
    problem_path: str
    solution_path: str
    problem_chars: int
    solution_chars: int
    problem_tokens: int
    code_tokens: int
    total_lines: int
    non_empty_lines: int
    code_lines: int
    windows_import_count: int
    windows_imports: List[str]
    windows_terminal_items: List[str]
    problem_hash_normalized: str
    solution_sha256: str


@dataclass
class DuplicateGroup:
    normalized_hash: str
    normalized_problem_preview: str
    kept_id: str
    dropped_ids: List[str]
    all_ids: List[str]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Prepare problem/solution Rust data for Qwen SFT.")
    parser.add_argument("--root", type=str, required=True, help="Dataset root containing problems/ and solutions/.")
    parser.add_argument("--problems-dir", type=str, default=None, help="Override problems directory. Default: <root>/problems")
    parser.add_argument("--solutions-dir", type=str, default=None, help="Override solutions directory. Default: <root>/solutions")
    parser.add_argument("--output-dir", type=str, required=True, help="Directory for dataset + analysis artifacts.")
    parser.add_argument(
        "--dataset-name",
        type=str,
        default="dataset.jsonl",
        help="Output dataset filename inside --output-dir. Default: dataset.jsonl",
    )
    parser.add_argument(
        "--model-name",
        type=str,
        default="Qwen/Qwen2.5-Coder-7B-Instruct",
        help="Tokenizer source for token counts.",
    )
    parser.add_argument("--local-files-only", action="store_true", help="Load tokenizer only from local HF cache.")
    parser.add_argument(
        "--system-prompt",
        type=str,
        default="",
        help="System prompt prepended to all rows.",
    )
    parser.add_argument(
        "--repair-system-prompt",
        type=str,
        default=None,
        help="System prompt for extra-dataset rows with type 'repair' (defaults to --system-prompt if omitted).",
    )
    parser.add_argument(
        "--critique-system-prompt",
        type=str,
        default=None,
        help="System prompt for extra-dataset rows with type 'critique' (defaults to --system-prompt if omitted).",
    )
    parser.add_argument(
        "--extra-datasets",
        action="append",
        default=[],
        help="Optional JSONL dataset path to merge (repeatable).",
    )
    parser.add_argument(
        "--conversational-datasets",
        action="append",
        default=[],
        help="Pre-formatted conversational JSONL path to merge without base-ID filtering (repeatable).",
    )
    parser.add_argument(
        "--max-conversational-fraction",
        type=float,
        default=1.0,
        help="Maximum fraction of final dataset that may be 'conversational' type (default 1.0, i.e. uncapped).",
    )
    parser.add_argument(
        "--secondary-roots",
        action="append",
        default=[],
        help="Additional dataset root directory containing problems/ and solutions/ sub-folders to merge into the main dataset (repeatable).",
    )
    parser.add_argument(
        "--problem-suffix",
        type=str,
        default=".md",
        help="Problem file extension. Default: .md",
    )
    parser.add_argument(
        "--solution-suffix",
        type=str,
        default=".rs",
        help="Solution file extension. Default: .rs",
    )
    parser.add_argument("--encoding", type=str, default="utf-8", help="Text encoding for input files.")
    parser.add_argument(
        "--keep-duplicates",
        action="store_true",
        help="Keep duplicate problems in the output dataset. They are still reported. Default filters them out.",
    )
    parser.add_argument(
        "--max-examples",
        type=int,
        default=None,
        help="Optional cap after pairing/sorting, useful for smoke tests.",
    )
    parser.add_argument(
        "--max-repair-fraction",
        type=float,
        default=0.15,
        help="Maximum fraction of final dataset that may be 'repair' type (default 0.15).",
    )
    parser.add_argument(
        "--max-critique-fraction",
        type=float,
        default=0.05,
        help="Maximum fraction of final dataset that may be 'critique' type (default 0.07).",
    )
    parser.add_argument(
        "--preview-top-n",
        type=int,
        default=20,
        help="How many top windows imports to print in the console summary.",
    )
    return parser.parse_args()


# -----------------------------
# File discovery / loading
# -----------------------------

def find_files(directory: Path, suffix: str) -> Dict[str, Path]:
    result: Dict[str, Path] = {}
    for path in sorted(directory.glob(f"*{suffix}")):
        if path.is_file():
            result[path.stem] = path
    return result



def read_text(path: Path, encoding: str) -> str:
    return path.read_text(encoding=encoding)



def pair_examples(
    problems_dir: Path,
    solutions_dir: Path,
    problem_suffix: str,
    solution_suffix: str,
    encoding: str,
) -> Tuple[List[ExampleRecord], List[str], List[str]]:
    problems = find_files(problems_dir, problem_suffix)
    solutions = find_files(solutions_dir, solution_suffix)

    missing_solutions = sorted(set(problems) - set(solutions))
    missing_problems = sorted(set(solutions) - set(problems))
    paired_ids = sorted(set(problems) & set(solutions))

    records: List[ExampleRecord] = []
    for sample_id in paired_ids:
        problem_path = problems[sample_id]
        solution_path = solutions[sample_id]
        records.append(
            ExampleRecord(
                sample_id=sample_id,
                problem_path=problem_path,
                solution_path=solution_path,
                problem_text=read_text(problem_path, encoding=encoding),
                solution_text=read_text(solution_path, encoding=encoding),
            )
        )

    return records, missing_solutions, missing_problems


# -----------------------------
# Problem normalization / dedupe
# -----------------------------

def strip_yaml_front_matter(text: str) -> str:
    if text.startswith("---\n"):
        parts = text.split("\n---\n", 1)
        if len(parts) == 2:
            return parts[1]
    return text



def normalize_markdown_for_dedupe(text: str) -> str:
    text = text.replace("\r\n", "\n").replace("\r", "\n")
    text = strip_yaml_front_matter(text)
    text = re.sub(r"<!--.*?-->", " ", text, flags=re.DOTALL)
    text = re.sub(r"\[([^\]]+)\]\(([^)]+)\)", r"\1", text)  # links -> label
    text = re.sub(r"```+", " ", text)
    text = re.sub(r"`([^`]*)`", r"\1", text)
    text = re.sub(r"^\s{0,3}#+\s*", "", text, flags=re.MULTILINE)
    text = re.sub(r"^\s*[-*+]\s+", "", text, flags=re.MULTILINE)
    text = re.sub(r"^\s*\d+[.)]\s+", "", text, flags=re.MULTILINE)
    text = re.sub(r"[*_~>]", " ", text)
    text = re.sub(r"<[^>]+>", " ", text)
    text = text.lower()
    text = re.sub(r"\s+", " ", text).strip()
    return text



def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()



def build_duplicate_groups(records: Sequence[ExampleRecord]) -> Tuple[List[DuplicateGroup], Dict[str, str]]:
    by_hash: Dict[str, List[ExampleRecord]] = defaultdict(list)
    normalized_by_id: Dict[str, str] = {}

    for record in records:
        normalized = normalize_markdown_for_dedupe(record.problem_text)
        normalized_hash = sha256_text(normalized)
        by_hash[normalized_hash].append(record)
        normalized_by_id[record.sample_id] = normalized_hash

    groups: List[DuplicateGroup] = []
    for normalized_hash, group_records in sorted(by_hash.items(), key=lambda kv: kv[0]):
        if len(group_records) <= 1:
            continue
        sorted_group = sorted(group_records, key=lambda r: r.sample_id)
        kept = sorted_group[0]
        dropped = sorted_group[1:]
        preview = normalize_markdown_for_dedupe(kept.problem_text)[:180]
        groups.append(
            DuplicateGroup(
                normalized_hash=normalized_hash,
                normalized_problem_preview=preview,
                kept_id=kept.sample_id,
                dropped_ids=[r.sample_id for r in dropped],
                all_ids=[r.sample_id for r in sorted_group],
            )
        )

    return groups, normalized_by_id



def filter_duplicate_records(records: Sequence[ExampleRecord], duplicate_groups: Sequence[DuplicateGroup]) -> List[ExampleRecord]:
    dropped_ids = {sample_id for group in duplicate_groups for sample_id in group.dropped_ids}
    return [record for record in records if record.sample_id not in dropped_ids]


# -----------------------------
# Rust analysis helpers
# -----------------------------

def count_loc(source: str) -> Tuple[int, int, int]:
    lines = source.splitlines()
    total_lines = len(lines)
    non_empty_lines = 0
    code_lines = 0
    in_block_comment = False

    for raw_line in lines:
        line = raw_line.rstrip("\n")
        stripped = line.strip()
        if stripped:
            non_empty_lines += 1

        i = 0
        has_code = False
        while i < len(line):
            if in_block_comment:
                end = line.find("*/", i)
                if end == -1:
                    i = len(line)
                    continue
                in_block_comment = False
                i = end + 2
                continue

            if line.startswith("//", i):
                break
            if line.startswith("/*", i):
                in_block_comment = True
                i += 2
                continue
            if not line[i].isspace():
                has_code = True
            i += 1

        if has_code:
            code_lines += 1

    return total_lines, non_empty_lines, code_lines



def split_top_level(text: str, delimiter: str = ",") -> List[str]:
    parts: List[str] = []
    depth = 0
    current: List[str] = []
    for ch in text:
        if ch == "{" and depth >= 0:
            depth += 1
        elif ch == "}" and depth > 0:
            depth -= 1
        if ch == delimiter and depth == 0:
            item = "".join(current).strip()
            if item:
                parts.append(item)
            current = []
        else:
            current.append(ch)
    tail = "".join(current).strip()
    if tail:
        parts.append(tail)
    return parts



def strip_alias(expr: str) -> str:
    expr = re.sub(r"\s+as\s+[A-Za-z_][A-Za-z0-9_]*\s*$", "", expr.strip())
    return expr.strip()



def find_matching_brace(text: str, open_index: int) -> int:
    depth = 0
    for idx in range(open_index, len(text)):
        ch = text[idx]
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return idx
    raise ValueError(f"Unmatched brace in use tree: {text!r}")



def join_path(prefix: str, suffix: str) -> str:
    prefix = prefix.strip().strip(":")
    suffix = suffix.strip().strip(":")
    if not prefix:
        return suffix
    if not suffix:
        return prefix
    return f"{prefix}::{suffix}"



def expand_use_tree(expr: str, base: str = "") -> List[str]:
    expr = strip_alias(expr.strip().rstrip(";"))
    if not expr:
        return []

    if "{" not in expr:
        if expr == "self":
            return [base] if base else []
        return [join_path(base, expr)]

    open_index = expr.find("{")
    close_index = find_matching_brace(expr, open_index)
    prefix = expr[:open_index].strip()
    inside = expr[open_index + 1 : close_index].strip()
    suffix_after = expr[close_index + 1 :].strip()

    if prefix.endswith("::"):
        prefix = prefix[:-2].strip()

    next_base = join_path(base, prefix) if prefix else base

    results: List[str] = []
    for part in split_top_level(inside, delimiter=","):
        results.extend(expand_use_tree(part, next_base))

    # Handles patterns like foo::{bar}::baz, which are rare in use trees.
    if suffix_after:
        suffix_after = suffix_after.lstrip(":")
        results = [join_path(item, suffix_after) for item in results]

    return results



def extract_use_statements(source: str) -> List[str]:
    source = re.sub(r"//.*?$", "", source, flags=re.MULTILINE)
    source = re.sub(r"/\*.*?\*/", " ", source, flags=re.DOTALL)
    statements = re.findall(r"\buse\s+([^;]+);", source, flags=re.DOTALL)
    return [stmt.strip() for stmt in statements]



def extract_windows_imports(source: str) -> List[str]:
    imports: List[str] = []
    for stmt in extract_use_statements(source):
        normalized_stmt = re.sub(r"\s+", " ", stmt)
        if not normalized_stmt.startswith("windows::"):
            continue
        expanded = expand_use_tree(normalized_stmt)
        imports.extend(path for path in expanded if path.startswith("windows::"))
    return sorted(imports)



def terminal_item(path: str) -> str:
    path = path.rstrip(":")
    return path.split("::")[-1] if path else path


# -----------------------------
# Serialization / reporting
# -----------------------------

def make_dataset_row(record: ExampleRecord, stats: FileStats, system_prompt: str) -> Dict[str, Any]:
    metadata: Dict[str, Any] = {
        "type": "base",
        "problem_path": stats.problem_path,
        "solution_path": stats.solution_path,
        "problem_tokens": stats.problem_tokens,
        "code_tokens": stats.code_tokens,
        "problem_chars": stats.problem_chars,
        "solution_chars": stats.solution_chars,
        "total_lines": stats.total_lines,
        "non_empty_lines": stats.non_empty_lines,
        "code_lines": stats.code_lines,
        "windows_import_count": stats.windows_import_count,
        "windows_imports": stats.windows_imports,
        "windows_terminal_items": stats.windows_terminal_items,
        "problem_hash_normalized": stats.problem_hash_normalized,
        "solution_sha256": stats.solution_sha256,
    }

    user_content = record.problem_text.strip()
    assistant_content = record.solution_text.rstrip() + "\n"
    return {
        "id": record.sample_id,
        "prompt": [
            {"role": "system", "content": system_prompt.strip()},
            {"role": "user", "content": user_content},
        ],
        "completion": [{"role": "assistant", "content": assistant_content}],
        "metadata": metadata,
    }


def load_extra_dataset(path: Path) -> List[Dict[str, Any]]:
    rows: List[Dict[str, Any]] = []
    with path.open("r", encoding="utf-8") as handle:
        for line_number, raw_line in enumerate(handle, start=1):
            line = raw_line.strip()
            if not line:
                continue
            try:
                payload = json.loads(line)
            except Exception:
                print(f"Warning: invalid JSON at {path}:{line_number}; skipping.", file=sys.stderr)
                continue
            if not isinstance(payload, dict):
                print(f"Warning: non-object row at {path}:{line_number}; skipping.", file=sys.stderr)
                continue
            if not isinstance(payload.get("id"), str) or not payload.get("id"):
                print(f"Warning: row missing id at {path}:{line_number}; skipping.", file=sys.stderr)
                continue
            rows.append(payload)
    return rows


def load_conversational_dataset(path: Path) -> List[Dict[str, Any]]:
    rows: List[Dict[str, Any]] = []

    def first_user_content(prompt: Any) -> str:
        if not isinstance(prompt, list):
            return ""
        for msg in prompt:
            if not isinstance(msg, dict):
                continue
            role = str(msg.get("role") or "").strip().lower()
            content = _to_text(msg.get("content")).strip()
            if role == "user" and content:
                return content
        return ""

    with path.open("r", encoding="utf-8") as handle:
        for line_number, raw_line in enumerate(handle, start=1):
            line = raw_line.strip()
            if not line:
                continue
            try:
                payload = json.loads(line)
            except Exception:
                print(f"Warning: invalid JSON at {path}:{line_number}; skipping.", file=sys.stderr)
                continue
            if not isinstance(payload, dict):
                print(f"Warning: non-object row at {path}:{line_number}; skipping.", file=sys.stderr)
                continue
            row_id = payload.get("id")
            has_id = isinstance(row_id, str) and row_id.strip()
            prompt_field = payload.get("prompt")
            has_prompt = isinstance(prompt_field, list) and len(prompt_field) > 0
            if not has_id and not has_prompt:
                print(f"Warning: row missing id and non-empty prompt at {path}:{line_number}; skipping.", file=sys.stderr)
                continue
            row = dict(payload)
            row["type"] = "conversational"
            if not has_id:
                user_text = first_user_content(prompt_field)
                if not user_text:
                    print(
                        f"Warning: row missing id and no user message in prompt at {path}:{line_number}; skipping.",
                        file=sys.stderr,
                    )
                    continue
                row["id"] = f"conv_{sha256_text(user_text)}"
            rows.append(row)
    return rows


def filter_by_base_ids(extra_rows: Sequence[Dict[str, Any]], base_ids: set[str]) -> Tuple[List[Dict[str, Any]], int]:
    kept: List[Dict[str, Any]] = []
    dropped = 0
    for row in extra_rows:
        problem_id = row.get("problem_id")
        row_id = row.get("id")
        match_key = problem_id if isinstance(problem_id, str) and problem_id else row_id
        if isinstance(match_key, str) and match_key in base_ids:
            kept.append(row)
        else:
            dropped += 1
    return kept, dropped


def cap_type_fraction(
    rows: List[Dict[str, Any]],
    type_name: str,
    max_fraction: float,
    rng: random.Random,
) -> List[Dict[str, Any]]:
    typed: List[Dict[str, Any]] = []
    others: List[Dict[str, Any]] = []
    for row in rows:
        metadata = row.get("metadata")
        row_type = metadata.get("type") if isinstance(metadata, dict) else None
        if row_type == type_name:
            typed.append(row)
        else:
            others.append(row)

    allowed = int(math.floor(max_fraction * len(rows)))
    if len(typed) <= allowed:
        return rows

    sampled_typed = rng.sample(typed, allowed)
    print(
        f"Warning: capped '{type_name}' rows from {len(typed)} to {allowed} "
        f"(max {max_fraction * 100:.1f}% of total)",
        file=sys.stderr,
    )
    return others + sampled_typed


def _to_text(value: Any) -> str:
    if value is None:
        return ""
    if isinstance(value, str):
        return value
    return json.dumps(value, ensure_ascii=False, indent=2)


def convert_to_conversational(
    row: Dict[str, Any],
    system_prompt: str,
    repair_system_prompt: str,
    critique_system_prompt: str,
) -> Dict[str, Any]:
    row_id = str(row.get("id") or "")
    row_type = str(row.get("type") or "").strip().lower()
    metadata = row.get("metadata") if isinstance(row.get("metadata"), dict) else {}
    merged_metadata: Dict[str, Any] = {"type": row_type or "extra", **metadata}

    def first_system_content(prompt: Any) -> str:
        if not isinstance(prompt, list):
            return ""
        for msg in prompt:
            if not isinstance(msg, dict):
                continue
            role = str(msg.get("role") or "").strip().lower()
            if role == "system":
                return _to_text(msg.get("content")).strip()
        return ""

    original_system = ""

    if row_type == "repair":
        user_content = (
            "Problem:\n"
            f"{_to_text(row.get('problem')).strip()}\n\n"
            "Failed code:\n"
            "```rust\n"
            f"{_to_text(row.get('failed_code')).rstrip()}\n"
            "```\n\n"
            "Feedback:\n"
            f"{_to_text(row.get('feedback')).strip()}"
        )
        assistant_content = _to_text(row.get("corrected_code")).rstrip() + "\n"
    elif row_type == "critique":
        critique_obj = row.get("critique")
        critique_text = _to_text(critique_obj if critique_obj is not None else row.get("diagnosis")).strip()
        user_content = (
            "Problem:\n"
            f"{_to_text(row.get('problem')).strip()}\n\n"
            "Candidate code:\n"
            "```rust\n"
            f"{_to_text(row.get('candidate_code')).rstrip()}\n"
            "```\n\n"
            "Feedback:\n"
            f"{_to_text(row.get('feedback')).strip()}"
        )
        assistant_content = critique_text
    elif isinstance(row.get("prompt"), list) and isinstance(row.get("completion"), list):
        prompt_rows = row.get("prompt") if isinstance(row.get("prompt"), list) else []
        completion_rows = row.get("completion") if isinstance(row.get("completion"), list) else []
        user_parts: List[str] = []
        assistant_parts: List[str] = []
        for msg in prompt_rows:
            if not isinstance(msg, dict):
                continue
            role = str(msg.get("role") or "").strip().lower()
            content = _to_text(msg.get("content")).strip()
            if role == "user" and content:
                user_parts.append(content)
        for msg in completion_rows:
            if not isinstance(msg, dict):
                continue
            role = str(msg.get("role") or "").strip().lower()
            content = _to_text(msg.get("content")).strip()
            if role == "assistant" and content:
                assistant_parts.append(content)
        user_content = "\n\n".join(user_parts).strip()
        assistant_content = "\n\n".join(assistant_parts).strip()
        original_system = first_system_content(prompt_rows)
    else:
        problem = _to_text(row.get("problem")).strip()
        failed_code = _to_text(row.get("failed_code")).strip()
        feedback = _to_text(row.get("feedback")).strip()
        if problem and failed_code:
            user_content = (
                "Problem:\n"
                f"{problem}\n\n"
                "Candidate code:\n"
                "```rust\n"
                f"{failed_code}\n"
                "```\n\n"
                "Feedback:\n"
                f"{feedback}"
            )
        elif problem:
            user_content = problem
        else:
            user_content = _to_text(row.get("input") or row.get("prompt") or "")
        assistant_content = _to_text(
            row.get("solution")
            or row.get("corrected_code")
            or row.get("output")
            or row.get("completion")
            or row.get("response")
            or ""
        ).strip()

    if row_type == "repair":
        effective_prompt = repair_system_prompt
    elif row_type == "critique":
        effective_prompt = critique_system_prompt
    else:
        effective_prompt = system_prompt

    if row_type == "conversational" and original_system:
        effective_prompt = original_system

    return {
        "id": row_id,
        "prompt": [
            {"role": "system", "content": effective_prompt.strip()},
            {"role": "user", "content": user_content.strip()},
        ],
        "completion": [{"role": "assistant", "content": assistant_content}],
        "metadata": merged_metadata,
    }



def safe_mean(values: Sequence[float]) -> float:
    return statistics.mean(values) if values else 0.0



def safe_median(values: Sequence[float]) -> float:
    return statistics.median(values) if values else 0.0



def percentile(values: Sequence[float], q: float) -> float:
    if not values:
        return 0.0
    if len(values) == 1:
        return float(values[0])
    sorted_values = sorted(float(v) for v in values)
    index = (len(sorted_values) - 1) * q
    lower = math.floor(index)
    upper = math.ceil(index)
    if lower == upper:
        return sorted_values[int(index)]
    weight = index - lower
    return sorted_values[lower] * (1 - weight) + sorted_values[upper] * weight



def write_json(path: Path, payload: Any) -> None:
    path.write_text(json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8")



def write_jsonl(path: Path, rows: Iterable[Dict[str, Any]]) -> None:
    with path.open("w", encoding="utf-8") as f:
        for row in rows:
            f.write(json.dumps(row, ensure_ascii=False) + "\n")



def write_csv(path: Path, fieldnames: List[str], rows: Iterable[Dict[str, Any]]) -> None:
    with path.open("w", encoding="utf-8", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        for row in rows:
            writer.writerow(row)



def summarize_numeric(values: Sequence[int]) -> Dict[str, float]:
    return {
        "min": min(values) if values else 0,
        "max": max(values) if values else 0,
        "mean": safe_mean(values),
        "median": safe_median(values),
        "p95": percentile(values, 0.95),
        "sum": sum(values),
    }



def main() -> None:
    args = parse_args()

    repair_system_prompt = args.repair_system_prompt if args.repair_system_prompt is not None else args.system_prompt
    critique_system_prompt = (
        args.critique_system_prompt if args.critique_system_prompt is not None else args.system_prompt
    )

    root = Path(args.root)
    problems_dir = Path(args.problems_dir) if args.problems_dir else root / "problems"
    solutions_dir = Path(args.solutions_dir) if args.solutions_dir else root / "solutions"
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    if not problems_dir.exists() or not problems_dir.is_dir():
        raise FileNotFoundError(f"Problems directory not found: {problems_dir}")
    if not solutions_dir.exists() or not solutions_dir.is_dir():
        raise FileNotFoundError(f"Solutions directory not found: {solutions_dir}")

    secondary_scan_targets: List[Tuple[Path, Path, Path]] = []
    for secondary_root_str in args.secondary_roots:
        sec_root = Path(secondary_root_str)
        sec_problems_dir = sec_root / "problems"
        sec_solutions_dir = sec_root / "solutions"
        if not sec_problems_dir.exists() or not sec_problems_dir.is_dir():
            print(f"Warning: secondary root missing problems directory: {sec_problems_dir}", file=sys.stderr)
            continue
        if not sec_solutions_dir.exists() or not sec_solutions_dir.is_dir():
            print(f"Warning: secondary root missing solutions directory: {sec_solutions_dir}", file=sys.stderr)
            continue
        secondary_scan_targets.append((sec_root, sec_problems_dir, sec_solutions_dir))

    print(f"Loading tokenizer: {args.model_name}")
    tokenizer = AutoTokenizer.from_pretrained(args.model_name, use_fast=True, local_files_only=args.local_files_only)

    print(f"Scanning dataset root: {root}")
    records, missing_solutions, missing_problems = pair_examples(
        problems_dir=problems_dir,
        solutions_dir=solutions_dir,
        problem_suffix=args.problem_suffix,
        solution_suffix=args.solution_suffix,
        encoding=args.encoding,
    )
    for sec_root, sec_problems_dir, sec_solutions_dir in secondary_scan_targets:
        print(f"Scanning secondary root: {sec_root}")
        sec_records, sec_missing_solutions, sec_missing_problems = pair_examples(
            problems_dir=sec_problems_dir,
            solutions_dir=sec_solutions_dir,
            problem_suffix=args.problem_suffix,
            solution_suffix=args.solution_suffix,
            encoding=args.encoding,
        )
        records.extend(sec_records)
        missing_solutions.extend(sec_missing_solutions)
        missing_problems.extend(sec_missing_problems)

    if args.max_examples is not None:
        records = records[: args.max_examples]

    duplicate_groups, normalized_hashes = build_duplicate_groups(records)

    if duplicate_groups:
        print("\nDuplicate problem warnings:", file=sys.stderr)
        for group in duplicate_groups:
            print(
                f"  - duplicate normalized problem hash {group.normalized_hash[:12]}... "
                f"keep={group.kept_id} drop={','.join(group.dropped_ids)}",
                file=sys.stderr,
            )
    else:
        print("\nNo duplicate problems detected.")

    filtered_records = records if args.keep_duplicates else filter_duplicate_records(records, duplicate_groups)

    file_stats_rows: List[FileStats] = []
    windows_full_counter: Counter[str] = Counter()
    windows_terminal_counter: Counter[str] = Counter()

    dataset_rows: List[Dict[str, Any]] = []
    for record in filtered_records:
        problem_text = record.problem_text.strip()
        solution_text = record.solution_text.rstrip() + "\n"

        problem_tokens = len(tokenizer(problem_text, add_special_tokens=False)["input_ids"])
        code_tokens = len(tokenizer(solution_text, add_special_tokens=False)["input_ids"])
        total_lines, non_empty_lines, code_lines = count_loc(solution_text)
        windows_imports = extract_windows_imports(solution_text)
        windows_terminal_items = [terminal_item(path) for path in windows_imports]

        for path in windows_imports:
            windows_full_counter[path] += 1
        for item in windows_terminal_items:
            windows_terminal_counter[item] += 1

        stats = FileStats(
            sample_id=record.sample_id,
            problem_path=str(record.problem_path),
            solution_path=str(record.solution_path),
            problem_chars=len(problem_text),
            solution_chars=len(solution_text),
            problem_tokens=problem_tokens,
            code_tokens=code_tokens,
            total_lines=total_lines,
            non_empty_lines=non_empty_lines,
            code_lines=code_lines,
            windows_import_count=len(windows_imports),
            windows_imports=windows_imports,
            windows_terminal_items=windows_terminal_items,
            problem_hash_normalized=normalized_hashes[record.sample_id],
            solution_sha256=sha256_text(solution_text),
        )
        file_stats_rows.append(stats)
        dataset_rows.append(
            make_dataset_row(
                record=record,
                stats=stats,
                system_prompt=args.system_prompt,
            )
        )

    base_ids = {record.sample_id for record in filtered_records}
    extra_dataset_summary: List[Dict[str, Any]] = []
    for dataset_path_str in args.extra_datasets:
        dataset_path = Path(dataset_path_str)
        if not dataset_path.exists():
            print(f"Warning: extra dataset not found: {dataset_path}", file=sys.stderr)
            extra_dataset_summary.append(
                {
                    "path": str(dataset_path),
                    "loaded": 0,
                    "kept": 0,
                    "dropped": 0,
                    "converted": 0,
                    "error": "not_found",
                }
            )
            continue

        loaded_rows = load_extra_dataset(dataset_path)
        kept_rows, dropped_count = filter_by_base_ids(loaded_rows, base_ids)
        converted_rows = [
            convert_to_conversational(row, args.system_prompt, repair_system_prompt, critique_system_prompt)
            for row in kept_rows
        ]
        dataset_rows.extend(converted_rows)
        extra_dataset_summary.append(
            {
                "path": str(dataset_path),
                "loaded": len(loaded_rows),
                "kept": len(kept_rows),
                "dropped": dropped_count,
                "converted": len(converted_rows),
            }
        )

    conversational_dataset_summary: List[Dict[str, Any]] = []
    for dataset_path_str in args.conversational_datasets:
        dataset_path = Path(dataset_path_str)
        if not dataset_path.exists():
            print(f"Warning: conversational dataset not found: {dataset_path}", file=sys.stderr)
            conversational_dataset_summary.append(
                {
                    "path": str(dataset_path),
                    "loaded": 0,
                    "converted": 0,
                    "error": "not_found",
                }
            )
            continue

        loaded_rows = load_conversational_dataset(dataset_path)
        converted_rows = [
            convert_to_conversational(row, args.system_prompt, repair_system_prompt, critique_system_prompt)
            for row in loaded_rows
        ]
        dataset_rows.extend(converted_rows)
        conversational_dataset_summary.append(
            {
                "path": str(dataset_path),
                "loaded": len(loaded_rows),
                "converted": len(converted_rows),
            }
        )

    rng = random.Random(42)
    dataset_rows = cap_type_fraction(dataset_rows, "conversational", args.max_conversational_fraction, rng)
    dataset_rows = cap_type_fraction(dataset_rows, "repair", args.max_repair_fraction, rng)
    dataset_rows = cap_type_fraction(dataset_rows, "critique", args.max_critique_fraction, rng)

    dataset_path = output_dir / args.dataset_name
    summary_path = output_dir / "summary.json"
    duplicates_path = output_dir / "duplicates.json"
    per_file_stats_path = output_dir / "per_file_stats.csv"
    windows_full_hist_path = output_dir / "windows_import_histogram_full_paths.csv"
    windows_terminal_hist_path = output_dir / "windows_import_histogram_terminal_items.csv"

    write_jsonl(dataset_path, dataset_rows)

    duplicate_payload = {
        "duplicate_group_count": len(duplicate_groups),
        "filtered_duplicate_sample_count": sum(len(group.dropped_ids) for group in duplicate_groups),
        "groups": [
            {
                "normalized_hash": group.normalized_hash,
                "normalized_problem_preview": group.normalized_problem_preview,
                "kept_id": group.kept_id,
                "dropped_ids": group.dropped_ids,
                "all_ids": group.all_ids,
            }
            for group in duplicate_groups
        ],
    }
    write_json(duplicates_path, duplicate_payload)

    per_file_csv_rows: List[Dict[str, Any]] = []
    for stats in file_stats_rows:
        per_file_csv_rows.append(
            {
                "id": stats.sample_id,
                "problem_path": stats.problem_path,
                "solution_path": stats.solution_path,
                "problem_chars": stats.problem_chars,
                "solution_chars": stats.solution_chars,
                "problem_tokens": stats.problem_tokens,
                "code_tokens": stats.code_tokens,
                "total_lines": stats.total_lines,
                "non_empty_lines": stats.non_empty_lines,
                "code_lines": stats.code_lines,
                "windows_import_count": stats.windows_import_count,
                "windows_imports": "; ".join(stats.windows_imports),
                "windows_terminal_items": "; ".join(stats.windows_terminal_items),
                "problem_hash_normalized": stats.problem_hash_normalized,
                "solution_sha256": stats.solution_sha256,
            }
        )
    write_csv(
        per_file_stats_path,
        fieldnames=list(per_file_csv_rows[0].keys()) if per_file_csv_rows else [
            "id",
            "problem_path",
            "solution_path",
            "problem_chars",
            "solution_chars",
            "problem_tokens",
            "code_tokens",
            "total_lines",
            "non_empty_lines",
            "code_lines",
            "windows_import_count",
            "windows_imports",
            "windows_terminal_items",
            "problem_hash_normalized",
            "solution_sha256",
        ],
        rows=per_file_csv_rows,
    )

    full_hist_rows = [
        {"windows_import_path": path, "occurrences": count}
        for path, count in sorted(windows_full_counter.items(), key=lambda kv: (-kv[1], kv[0]))
    ]
    terminal_hist_rows = [
        {"windows_item": item, "occurrences": count}
        for item, count in sorted(windows_terminal_counter.items(), key=lambda kv: (-kv[1], kv[0]))
    ]
    write_csv(
        windows_full_hist_path,
        fieldnames=["windows_import_path", "occurrences"],
        rows=full_hist_rows,
    )
    write_csv(
        windows_terminal_hist_path,
        fieldnames=["windows_item", "occurrences"],
        rows=terminal_hist_rows,
    )

    problem_token_values = [row.problem_tokens for row in file_stats_rows]
    code_token_values = [row.code_tokens for row in file_stats_rows]
    total_line_values = [row.total_lines for row in file_stats_rows]
    non_empty_line_values = [row.non_empty_lines for row in file_stats_rows]
    code_line_values = [row.code_lines for row in file_stats_rows]
    repair_rows_in_output = sum(
        1
        for row in dataset_rows
        if isinstance(row.get("metadata"), dict) and row["metadata"].get("type") == "repair"
    )
    critique_rows_in_output = sum(
        1
        for row in dataset_rows
        if isinstance(row.get("metadata"), dict) and row["metadata"].get("type") == "critique"
    )
    conversational_rows_in_output = sum(
        1
        for row in dataset_rows
        if isinstance(row.get("metadata"), dict) and row["metadata"].get("type") == "conversational"
    )

    summary = {
        "input": {
            "root": str(root),
            "problems_dir": str(problems_dir),
            "solutions_dir": str(solutions_dir),
            "secondary_roots": [str(sec_root) for sec_root, _, _ in secondary_scan_targets],
            "problem_suffix": args.problem_suffix,
            "solution_suffix": args.solution_suffix,
            "encoding": args.encoding,
            "tokenizer": args.model_name,
            "output_format": "conversational",
            "repair_system_prompt": repair_system_prompt,
            "critique_system_prompt": critique_system_prompt,
        },
        "counts": {
            "paired_examples_before_dedup": len(records),
            "paired_examples_after_dedup": len(filtered_records),
            "missing_solutions": len(missing_solutions),
            "missing_problems": len(missing_problems),
            "duplicate_problem_groups": len(duplicate_groups),
            "filtered_duplicate_samples": 0 if args.keep_duplicates else sum(len(group.dropped_ids) for group in duplicate_groups),
            "extra_rows_added": sum(item.get("converted", 0) for item in extra_dataset_summary),
            "conversational_rows_added": sum(item.get("converted", 0) for item in conversational_dataset_summary),
            "total_dataset_rows": len(dataset_rows),
            "repair_rows_in_output": repair_rows_in_output,
            "critique_rows_in_output": critique_rows_in_output,
            "conversational_rows_in_output": conversational_rows_in_output,
        },
        "missing_pairs": {
            "missing_solutions_ids": missing_solutions,
            "missing_problems_ids": missing_problems,
        },
        "tokens": {
            "problem_tokens": summarize_numeric(problem_token_values),
            "code_tokens": summarize_numeric(code_token_values),
            "combined_tokens_total": sum(problem_token_values) + sum(code_token_values),
            "total_code_tokens": sum(code_token_values),
            "total_problem_tokens": sum(problem_token_values),
        },
        "lines": {
            "total_lines": summarize_numeric(total_line_values),
            "non_empty_lines": summarize_numeric(non_empty_line_values),
            "code_lines": summarize_numeric(code_line_values),
        },
        "windows_imports": {
            "unique_full_paths": len(windows_full_counter),
            "unique_terminal_items": len(windows_terminal_counter),
            "top_full_paths": full_hist_rows[:100],
            "top_terminal_items": terminal_hist_rows[:100],
        },
        "extra_datasets": extra_dataset_summary,
        "conversational_datasets": conversational_dataset_summary,
        "artifacts": {
            "dataset_jsonl": str(dataset_path),
            "summary_json": str(summary_path),
            "duplicates_json": str(duplicates_path),
            "per_file_stats_csv": str(per_file_stats_path),
            "windows_import_histogram_full_paths_csv": str(windows_full_hist_path),
            "windows_import_histogram_terminal_items_csv": str(windows_terminal_hist_path),
        },
    }
    write_json(summary_path, summary)

    print("\nPreparation complete.")
    print(f"Output dataset: {dataset_path}")
    print(f"Examples kept: {len(filtered_records)} / {len(records)}")
    print(f"Missing problems: {len(missing_problems)}")
    print(f"Missing solutions: {len(missing_solutions)}")
    print(f"Duplicate groups: {len(duplicate_groups)}")
    print(f"Extra rows added: {sum(item.get('converted', 0) for item in extra_dataset_summary)}")
    print(f"Conversational rows added: {sum(item.get('converted', 0) for item in conversational_dataset_summary)}")
    print(f"Final dataset rows: {len(dataset_rows)}")
    print(
        f"Repair rows: {repair_rows_in_output} / {len(dataset_rows)} "
        f"({(repair_rows_in_output / len(dataset_rows) * 100) if dataset_rows else 0.0:.2f}%)"
    )
    print(
        f"Critique rows: {critique_rows_in_output} / {len(dataset_rows)} "
        f"({(critique_rows_in_output / len(dataset_rows) * 100) if dataset_rows else 0.0:.2f}%)"
    )
    print(f"Total code tokens: {summary['tokens']['total_code_tokens']}")
    print("\nTop windows imports (full paths):")
    for row in full_hist_rows[: args.preview_top_n]:
        print(f"  {row['occurrences']:>6}  {row['windows_import_path']}")

    print(
        "\nTraining command example:\n"
        f"  accelerate launch train_qwen_rust_sft.py --train-file {dataset_path} "
        f"--output-dir outputs/qwen25-coder-rust-7b-qlora --use-4bit"
    )


if __name__ == "__main__":
    main()
