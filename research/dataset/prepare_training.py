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

Output dataset formats:
1) pair (default)
   {
     "id": "...",
     "problem": "...",
     "solution": "...",
     "metadata": {...}
   }

2) conversational
   {
     "id": "...",
     "prompt": [{"role": "system", "content": "..."}, {"role": "user", "content": "..."}],
     "completion": [{"role": "assistant", "content": "..."}],
     "metadata": {...}
   }

Example:
    python prepare_training.py ^
      --root data ^
      --output-dir prepared ^
      --model-name Qwen/Qwen2.5-Coder-7B-Instruct ^
      --output-format pair ^
      --system-prompt "You are a precise Rust coding assistant. Return correct, idiomatic Rust."
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import math
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
        "--output-format",
        type=str,
        default="pair",
        choices=["pair", "conversational"],
        help="pair -> fields problem/solution; conversational -> prompt/completion",
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
        help="Optional system prompt when --output-format conversational is used.",
    )
    parser.add_argument(
        "--problem-field",
        type=str,
        default="problem",
        help="Field name for problem text when --output-format pair is used.",
    )
    parser.add_argument(
        "--solution-field",
        type=str,
        default="solution",
        help="Field name for solution text when --output-format pair is used.",
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

def make_dataset_row(record: ExampleRecord, stats: FileStats, output_format: str, problem_field: str, solution_field: str, system_prompt: str) -> Dict[str, Any]:
    metadata: Dict[str, Any] = {
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

    row: Dict[str, Any] = {"id": record.sample_id, "metadata": metadata}
    if output_format == "pair":
        row[problem_field] = record.problem_text.strip()
        row[solution_field] = record.solution_text.rstrip() + "\n"
    else:
        prompt: List[Dict[str, str]] = []
        if system_prompt.strip():
            prompt.append({"role": "system", "content": system_prompt.strip()})
        prompt.append({"role": "user", "content": record.problem_text.strip()})
        completion = [{"role": "assistant", "content": record.solution_text.rstrip() + "\n"}]
        row["prompt"] = prompt
        row["completion"] = completion
    return row



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

    root = Path(args.root)
    problems_dir = Path(args.problems_dir) if args.problems_dir else root / "problems"
    solutions_dir = Path(args.solutions_dir) if args.solutions_dir else root / "solutions"
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    if not problems_dir.exists() or not problems_dir.is_dir():
        raise FileNotFoundError(f"Problems directory not found: {problems_dir}")
    if not solutions_dir.exists() or not solutions_dir.is_dir():
        raise FileNotFoundError(f"Solutions directory not found: {solutions_dir}")

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
                output_format=args.output_format,
                problem_field=args.problem_field,
                solution_field=args.solution_field,
                system_prompt=args.system_prompt,
            )
        )

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

    summary = {
        "input": {
            "root": str(root),
            "problems_dir": str(problems_dir),
            "solutions_dir": str(solutions_dir),
            "problem_suffix": args.problem_suffix,
            "solution_suffix": args.solution_suffix,
            "encoding": args.encoding,
            "tokenizer": args.model_name,
            "output_format": args.output_format,
        },
        "counts": {
            "paired_examples_before_dedup": len(records),
            "paired_examples_after_dedup": len(filtered_records),
            "missing_solutions": len(missing_solutions),
            "missing_problems": len(missing_problems),
            "duplicate_problem_groups": len(duplicate_groups),
            "filtered_duplicate_samples": 0 if args.keep_duplicates else sum(len(group.dropped_ids) for group in duplicate_groups),
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
    print(f"Total code tokens: {summary['tokens']['total_code_tokens']}")
    print("\nTop windows imports (full paths):")
    for row in full_hist_rows[: args.preview_top_n]:
        print(f"  {row['occurrences']:>6}  {row['windows_import_path']}")

    if args.output_format == "pair":
        print(
            "\nTraining command example:\n"
            f"  accelerate launch train_qwen_rust_sft.py --train-file {dataset_path} "
            f"--output-dir outputs/qwen25-coder-rust-7b-qlora "
            f"--user-field {args.problem_field} --assistant-field {args.solution_field} --use-4bit"
        )
    else:
        print(
            "\nTraining command example:\n"
            f"  accelerate launch train_qwen_rust_sft.py --train-file {dataset_path} "
            f"--output-dir outputs/qwen25-coder-rust-7b-qlora --use-4bit"
        )


if __name__ == "__main__":
    main()
