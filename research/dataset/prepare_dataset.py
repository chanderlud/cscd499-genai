#!/usr/bin/env python3
"""
Build a dataset JSON file by pairing problem prompts (.md) with tests (.rs)
based on matching UUID filenames.

Output JSON format:
{
  "<uuid>": { "prompt": "...", "tests": "..." },
  ...
}
"""

from __future__ import annotations

import argparse
import json
import sys
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Tuple


@dataclass
class ProblemContents:
    prompt: str
    tests: str


def is_uuid(stem: str) -> bool:
    try:
        uuid.UUID(stem)
        return True
    except ValueError:
        return False


def index_dir(dir_path: Path, ext: str, validate_uuid: bool) -> Dict[str, Path]:
    """
    Return mapping: filename_stem -> file_path for all files matching `*{ext}`.
    """
    if not dir_path.exists() or not dir_path.is_dir():
        raise FileNotFoundError(f"Not a directory: {dir_path}")

    out: Dict[str, Path] = {}
    for p in dir_path.iterdir():
        if not p.is_file():
            continue
        if p.suffix.lower() != ext.lower():
            continue

        stem = p.stem
        if validate_uuid and not is_uuid(stem):
            continue

        # If duplicates exist, fail loudly. Better than silently picking one.
        if stem in out:
            raise ValueError(f"Duplicate {ext} filename stem '{stem}' in {dir_path}")
        out[stem] = p

    return out


def read_text(path: Path) -> str:
    # utf-8-sig handles BOM if present, without breaking normal utf-8 files
    return path.read_text(encoding="utf-8-sig")


def build_dataset(
        problems_dir: Path,
        tests_dir: Path,
        strict: bool,
        validate_uuid: bool,
) -> Tuple[Dict[str, ProblemContents], Dict[str, Path], Dict[str, Path]]:
    problems = index_dir(problems_dir, ".md", validate_uuid=validate_uuid)
    tests = index_dir(tests_dir, ".rs", validate_uuid=validate_uuid)

    problem_keys = set(problems.keys())
    test_keys = set(tests.keys())

    missing_tests = sorted(problem_keys - test_keys)
    missing_problems = sorted(test_keys - problem_keys)
    common = sorted(problem_keys & test_keys)

    if strict and (missing_tests or missing_problems):
        msg_lines = []
        if missing_tests:
            msg_lines.append(f"Missing tests for {len(missing_tests)} problem(s): {missing_tests[:10]}")
            if len(missing_tests) > 10:
                msg_lines.append("  ... (truncated)")
        if missing_problems:
            msg_lines.append(f"Missing problems for {len(missing_problems)} test(s): {missing_problems[:10]}")
            if len(missing_problems) > 10:
                msg_lines.append("  ... (truncated)")
        raise RuntimeError("\n".join(msg_lines))

    dataset: Dict[str, ProblemContents] = {}
    for k in common:
        dataset[k] = ProblemContents(
            prompt=read_text(problems[k]),
            tests=read_text(tests[k]),
        )

    return dataset, problems, tests


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Prepare a dataset JSON by pairing problems/<UUID>.md with tests/<UUID>.rs"
    )
    parser.add_argument("--problems", required=True, type=Path, help="Directory containing .md prompt files")
    parser.add_argument("--tests", required=True, type=Path, help="Directory containing .rs test files")
    parser.add_argument("--out", required=True, type=Path, help="Output JSON path")
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Fail if any problem is missing tests or any test is missing a problem",
    )
    parser.add_argument(
        "--no-uuid-validate",
        action="store_true",
        help="Do not filter filenames by UUID validity (use all .md/.rs stems)",
    )
    parser.add_argument(
        "--pretty",
        action="store_true",
        help="Pretty-print JSON (indented).",
    )

    args = parser.parse_args()

    validate_uuid = not args.no_uuid_validate

    try:
        dataset, problems_map, tests_map = build_dataset(
            problems_dir=args.problems,
            tests_dir=args.tests,
            strict=args.strict,
            validate_uuid=validate_uuid,
        )
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 2

    # Convert dataclasses to plain dicts for JSON serialization
    json_obj = {k: {"prompt": v.prompt, "tests": v.tests} for k, v in dataset.items()}

    args.out.parent.mkdir(parents=True, exist_ok=True)
    with args.out.open("w", encoding="utf-8") as f:
        json.dump(
            json_obj,
            f,
            ensure_ascii=False,
            indent=2 if args.pretty else None,
            sort_keys=True,
        )
        f.write("\n")

    # Non-strict mode: print a useful summary to stderr
    problem_keys = set(problems_map.keys())
    test_keys = set(tests_map.keys())
    missing_tests = sorted(problem_keys - test_keys)
    missing_problems = sorted(test_keys - problem_keys)

    print(f"Wrote {len(dataset)} paired item(s) to {args.out}", file=sys.stderr)
    if missing_tests:
        print(f"Warning: {len(missing_tests)} problem(s) missing tests (skipped)", file=sys.stderr)
    if missing_problems:
        print(f"Warning: {len(missing_problems)} test(s) missing problems (skipped)", file=sys.stderr)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())