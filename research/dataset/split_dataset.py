#!/usr/bin/env python
"""
Partition paired problems/solutions/tests datasets into train/val/test splits.

Input layout per root:
    <root>/problems/<uuid>.md
    <root>/solutions/<uuid>.rs
    <root>/tests/<uuid>.rs    # optional directory, optional per-sample file

You may provide one or more dataset roots. The script will collect all fully
paired problem/solution files across those roots before splitting.

Output layout:
    <output>/train/problems/<uuid>.md
    <output>/train/solutions/<uuid>.rs
    <output>/train/tests/<uuid>.rs      # only for samples that have tests
    <output>/val/...
    <output>/test/...

Important test-split rule:
- Only samples with an associated tests/<id>.rs file are eligible for the test
  split.
- If a root has no tests/ directory, samples from that root may only be placed
  into train or val.
- When duplicate-problem grouping is enabled, an entire duplicate group must be
  test-eligible before that group can be assigned to the test split.

Features:
- Collects paired samples from one or more roots.
- Pairs files by matching basename stem (UUID or any shared basename).
- Warns about missing files in problems/, solutions/, and tests/.
- Errors if the same fully paired basename appears in multiple roots, because
  that would collide in the output layout.
- Default split is tuned for ~100 samples: 80/10/10.
- Optional duplicate-problem grouping keeps exact normalized duplicate prompts in
  the same split to reduce train/val/test leakage.
- Supports copy, hardlink, or move output actions.
- Writes manifests and a summary JSON report.

Examples:
    python split_dataset.py ^
      --root data_a ^
      --root data_b ^
      --output-dir splits

    python split_dataset.py ^
      --root data_a data_b ^
      --output-dir splits ^
      --seed 1337 ^
      --action hardlink
"""

from __future__ import annotations

import argparse
import json
import math
import os
import random
import re
import shutil
import sys
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Iterable, List, Sequence, Tuple


@dataclass(frozen=True)
class SampleRecord:
    sample_id: str
    source_root: str
    source_stem: str
    problem_path: Path
    solution_path: Path
    test_path: Path | None
    problem_text: str

    @property
    def has_test(self) -> bool:
        return self.test_path is not None


@dataclass(frozen=True)
class GroupRecord:
    group_key: str
    member_ids: List[str]
    test_eligible: bool



def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Partition paired problem/solution files into train/val/test splits.")
    parser.add_argument(
        "--root",
        dest="roots",
        action="append",
        nargs="+",
        required=True,
        help="Dataset root containing problems/ and solutions/. tests/ is optional. Can be passed multiple times or with multiple paths.",
    )
    parser.add_argument(
        "--problems-dir",
        type=str,
        default="problems",
        help="Problems subdirectory name under each root. Default: problems",
    )
    parser.add_argument(
        "--solutions-dir",
        type=str,
        default="solutions",
        help="Solutions subdirectory name under each root. Default: solutions",
    )
    parser.add_argument(
        "--tests-dir",
        type=str,
        default="tests",
        help="Optional tests subdirectory name under each root. Default: tests",
    )
    parser.add_argument("--output-dir", type=str, required=True, help="Output directory for split folders + manifests.")
    parser.add_argument("--problem-suffix", type=str, default=".md", help="Problem file extension. Default: .md")
    parser.add_argument("--solution-suffix", type=str, default=".rs", help="Solution file extension. Default: .rs")
    parser.add_argument("--test-suffix", type=str, default=".rs", help="Test file extension. Default: .rs")
    parser.add_argument("--encoding", type=str, default="utf-8", help="File text encoding. Default: utf-8")
    parser.add_argument("--seed", type=int, default=42, help="Shuffle seed. Default: 42")
    parser.add_argument(
        "--action",
        type=str,
        choices=["copy", "hardlink", "move"],
        default="copy",
        help="How to place files into split directories. Default: copy",
    )
    parser.add_argument(
        "--train-ratio",
        type=float,
        default=0.80,
        help="Train ratio. Default 0.80, chosen so 100 samples -> 80 train.",
    )
    parser.add_argument(
        "--val-ratio",
        type=float,
        default=0.10,
        help="Validation ratio. Default 0.10, so 100 samples -> 10 val.",
    )
    parser.add_argument(
        "--test-ratio",
        type=float,
        default=0.10,
        help="Test ratio. Default 0.10, so 100 samples -> 10 test when enough test-eligible samples exist.",
    )
    parser.add_argument(
        "--split-counts",
        type=int,
        nargs=3,
        metavar=("TRAIN", "VAL", "TEST"),
        default=None,
        help="Use exact split counts instead of ratios.",
    )
    parser.add_argument(
        "--group-duplicates",
        action="store_true",
        default=True,
        help="Keep exact normalized duplicate problems in the same split. Default: enabled.",
    )
    parser.add_argument(
        "--no-group-duplicates",
        dest="group_duplicates",
        action="store_false",
        help="Disable duplicate-problem grouping.",
    )
    parser.add_argument(
        "--overwrite",
        action="store_true",
        help="Allow writing into a non-empty output directory.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Compute split/manifests/summary without copying, linking, or moving files.",
    )
    return parser.parse_args()



def flatten_roots(root_args: Sequence[Sequence[str]]) -> List[Path]:
    roots: List[Path] = []
    seen: set[Path] = set()

    for group in root_args:
        for raw_root in group:
            root = Path(raw_root)
            normalized = root.resolve()
            if normalized in seen:
                continue
            seen.add(normalized)
            roots.append(root)

    return roots



def find_files(directory: Path, suffix: str) -> Dict[str, Path]:
    result: Dict[str, Path] = {}
    for path in sorted(directory.glob(f"*{suffix}")):
        if path.is_file():
            result[path.stem] = path
    return result



def read_text(path: Path, encoding: str) -> str:
    return path.read_text(encoding=encoding)



def scoped_id(root: Path, sample_id: str) -> str:
    return f"{root}::{sample_id}"



def pair_records_in_root(
        root: Path,
        problems_dir: Path,
        solutions_dir: Path,
        tests_dir: Path,
        problem_suffix: str,
        solution_suffix: str,
        test_suffix: str,
        encoding: str,
) -> Tuple[List[SampleRecord], Dict[str, List[str]]]:
    problems = find_files(problems_dir, problem_suffix)
    solutions = find_files(solutions_dir, solution_suffix)

    tests_dir_present = tests_dir.exists()
    if tests_dir_present and not tests_dir.is_dir():
        raise FileNotFoundError(f"tests path exists but is not a directory: {tests_dir}")
    tests = find_files(tests_dir, test_suffix) if tests_dir_present else {}

    all_ids = set(problems) | set(solutions) | set(tests)
    paired_ids = sorted(set(problems) & set(solutions))

    missing = {
        "missing_problem": sorted(scoped_id(root, sample_id) for sample_id in ((set(solutions) | set(tests)) - set(problems))),
        "missing_solution": sorted(scoped_id(root, sample_id) for sample_id in ((set(problems) | set(tests)) - set(solutions))),
        "missing_test": sorted(scoped_id(root, sample_id) for sample_id in (set(paired_ids) - set(tests))) if tests_dir_present else [],
        "unpaired_any": sorted(scoped_id(root, sample_id) for sample_id in (all_ids - set(paired_ids))),
        "roots_without_tests_dir": [] if tests_dir_present else [str(root)],
    }

    records: List[SampleRecord] = []
    for sample_id in paired_ids:
        records.append(
            SampleRecord(
                sample_id=sample_id,
                source_root=str(root),
                source_stem=sample_id,
                problem_path=problems[sample_id],
                solution_path=solutions[sample_id],
                test_path=tests.get(sample_id),
                problem_text=read_text(problems[sample_id], encoding=encoding),
            )
        )

    return records, missing



def collect_records(
        roots: Sequence[Path],
        problems_subdir: str,
        solutions_subdir: str,
        tests_subdir: str,
        problem_suffix: str,
        solution_suffix: str,
        test_suffix: str,
        encoding: str,
) -> Tuple[List[SampleRecord], Dict[str, List[str]]]:
    all_records: List[SampleRecord] = []
    aggregated_missing: Dict[str, List[str]] = {
        "missing_problem": [],
        "missing_solution": [],
        "missing_test": [],
        "unpaired_any": [],
        "roots_without_tests_dir": [],
    }
    paired_id_sources: Dict[str, List[str]] = defaultdict(list)

    for root in roots:
        problems_dir = root / problems_subdir
        solutions_dir = root / solutions_subdir
        tests_dir = root / tests_subdir

        if not problems_dir.exists() or not problems_dir.is_dir():
            raise FileNotFoundError(f"required directory does not exist or is not a directory: {problems_dir}")
        if not solutions_dir.exists() or not solutions_dir.is_dir():
            raise FileNotFoundError(f"required directory does not exist or is not a directory: {solutions_dir}")

        root_records, root_missing = pair_records_in_root(
            root=root,
            problems_dir=problems_dir,
            solutions_dir=solutions_dir,
            tests_dir=tests_dir,
            problem_suffix=problem_suffix,
            solution_suffix=solution_suffix,
            test_suffix=test_suffix,
            encoding=encoding,
        )
        all_records.extend(root_records)
        for key in aggregated_missing:
            aggregated_missing[key].extend(root_missing[key])

        for record in root_records:
            paired_id_sources[record.sample_id].append(record.source_root)

    duplicate_paired_ids = {
        sample_id: sorted(source_roots)
        for sample_id, source_roots in paired_id_sources.items()
        if len(source_roots) > 1
    }
    if duplicate_paired_ids:
        preview = []
        for sample_id, source_roots in list(sorted(duplicate_paired_ids.items()))[:10]:
            preview.append(f"{sample_id}: {', '.join(source_roots)}")
        more = "" if len(duplicate_paired_ids) <= 10 else f" ... (+{len(duplicate_paired_ids) - 10} more)"
        raise ValueError(
            "Duplicate fully paired sample ids were found across roots. "
            "Output filenames would collide. Offending ids: "
            + " | ".join(preview)
            + more
        )

    all_records.sort(key=lambda record: record.sample_id)
    for key in aggregated_missing:
        aggregated_missing[key] = sorted(aggregated_missing[key])

    return all_records, aggregated_missing


# Duplicate normalization matches the prep script closely enough to be useful.
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
    text = re.sub(r"\[([^\]]+)\]\(([^)]+)\)", r"\1", text)
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



def build_groups(records: Sequence[SampleRecord], group_duplicates: bool, seed: int) -> Tuple[List[GroupRecord], Dict[str, Dict[str, object]]]:
    by_id = {record.sample_id: record for record in records}

    if not group_duplicates:
        groups = [
            GroupRecord(group_key=record.sample_id, member_ids=[record.sample_id], test_eligible=record.has_test)
            for record in records
        ]
        return groups, {}

    by_hash: Dict[str, List[str]] = defaultdict(list)
    previews: Dict[str, str] = {}

    for record in records:
        normalized = normalize_markdown_for_dedupe(record.problem_text)
        key = normalized if normalized else f"EMPTY::{record.sample_id}"
        by_hash[key].append(record.sample_id)
        if key not in previews:
            previews[key] = normalized[:160]

    rng = random.Random(seed)
    groups: List[GroupRecord] = []
    duplicate_report: Dict[str, Dict[str, object]] = {}

    for group_key, member_ids in by_hash.items():
        member_ids = sorted(member_ids)
        test_eligible = all(by_id[sample_id].has_test for sample_id in member_ids)
        groups.append(GroupRecord(group_key=group_key, member_ids=member_ids, test_eligible=test_eligible))
        if len(member_ids) > 1:
            duplicate_report[group_key] = {
                "size": len(member_ids),
                "member_ids": member_ids,
                "preview": previews[group_key],
                "test_eligible": test_eligible,
            }

    rng.shuffle(groups)
    groups.sort(key=lambda g: len(g.member_ids), reverse=True)
    return groups, duplicate_report



def distribute_remainder(base: Dict[str, int], raw: Dict[str, float], keys: Sequence[str], remainder: int) -> None:
    order = sorted(keys, key=lambda name: (raw[name] - base[name], raw[name]), reverse=True)
    for i in range(remainder):
        base[order[i % len(order)]] += 1



def derive_target_counts(
        total: int,
        test_eligible_total: int,
        train_ratio: float,
        val_ratio: float,
        test_ratio: float,
        split_counts: Sequence[int] | None,
) -> Dict[str, int]:
    if split_counts is not None:
        train_count, val_count, test_count = split_counts
        if train_count < 0 or val_count < 0 or test_count < 0:
            raise ValueError("Split counts must be non-negative.")
        if train_count + val_count + test_count != total:
            raise ValueError(
                f"Explicit split counts must sum to total paired samples ({total}), got {train_count + val_count + test_count}."
            )
        if test_count > test_eligible_total:
            raise ValueError(
                f"Requested test split count ({test_count}) exceeds the number of test-eligible samples ({test_eligible_total})."
            )
        return {"train": train_count, "val": val_count, "test": test_count}

    ratio_sum = train_ratio + val_ratio + test_ratio
    if not math.isclose(ratio_sum, 1.0, rel_tol=1e-6, abs_tol=1e-6):
        raise ValueError(f"Ratios must sum to 1.0, got {ratio_sum:.8f}")

    raw = {
        "train": total * train_ratio,
        "val": total * val_ratio,
        "test": total * test_ratio,
    }
    counts = {name: int(math.floor(value)) for name, value in raw.items()}
    remainder = total - sum(counts.values())
    distribute_remainder(counts, raw, ["train", "val", "test"], remainder)

    if counts["test"] <= test_eligible_total:
        return counts

    overflow = counts["test"] - test_eligible_total
    counts["test"] = test_eligible_total

    train_val_ratio_sum = train_ratio + val_ratio
    if train_val_ratio_sum <= 0:
        raise ValueError("Train and validation ratios cannot both be zero when test capacity is limited.")

    raw_rebalance = {
        "train": overflow * (train_ratio / train_val_ratio_sum),
        "val": overflow * (val_ratio / train_val_ratio_sum),
    }
    rebalance = {name: int(math.floor(value)) for name, value in raw_rebalance.items()}
    rebalance_remainder = overflow - sum(rebalance.values())
    distribute_remainder(rebalance, raw_rebalance, ["train", "val"], rebalance_remainder)

    counts["train"] += rebalance["train"]
    counts["val"] += rebalance["val"]
    return counts



def assign_groups_to_splits(groups: Sequence[GroupRecord], target_counts: Dict[str, int], seed: int) -> Dict[str, List[str]]:
    rng = random.Random(seed)
    split_names = ["train", "val", "test"]
    assignments: Dict[str, List[str]] = {name: [] for name in split_names}
    current_counts: Dict[str, int] = {name: 0 for name in split_names}

    groups = list(groups)
    rng.shuffle(groups)
    groups.sort(key=lambda g: len(g.member_ids), reverse=True)

    for group in groups:
        group_size = len(group.member_ids)
        allowed_splits = ["train", "val", "test"] if group.test_eligible else ["train", "val"]

        ranked_splits = sorted(
            allowed_splits,
            key=lambda name: (
                target_counts[name] - current_counts[name],
                target_counts[name],
                1 if name == "train" else 0,
                1 if name == "val" else 0,
            ),
            reverse=True,
        )

        chosen = None
        for name in ranked_splits:
            if current_counts[name] + group_size <= target_counts[name]:
                chosen = name
                break

        if chosen is None:
            chosen = min(
                allowed_splits,
                key=lambda name: (
                    abs((current_counts[name] + group_size) - target_counts[name]),
                    current_counts[name] - target_counts[name],
                    0 if name == "train" else 1,
                    0 if name == "val" else 1,
                ),
            )

        assignments[chosen].extend(group.member_ids)
        current_counts[chosen] += group_size

    for name in split_names:
        assignments[name] = sorted(assignments[name])

    return assignments



def ensure_output_dir(output_dir: Path, overwrite: bool) -> None:
    if output_dir.exists():
        if any(output_dir.iterdir()) and not overwrite:
            raise FileExistsError(
                f"Output directory '{output_dir}' already exists and is not empty. Use --overwrite to allow this."
            )
    output_dir.mkdir(parents=True, exist_ok=True)



def materialize_file(src: Path, dst: Path, action: str) -> None:
    dst.parent.mkdir(parents=True, exist_ok=True)
    if action == "copy":
        shutil.copy2(src, dst)
    elif action == "hardlink":
        if dst.exists():
            dst.unlink()
        os.link(src, dst)
    elif action == "move":
        shutil.move(str(src), str(dst))
    else:
        raise ValueError(f"Unsupported action: {action}")



def write_json(path: Path, obj: object) -> None:
    path.write_text(json.dumps(obj, indent=2, ensure_ascii=False), encoding="utf-8")



def write_jsonl(path: Path, rows: Iterable[dict]) -> None:
    with path.open("w", encoding="utf-8") as f:
        for row in rows:
            f.write(json.dumps(row, ensure_ascii=False) + "\n")



def build_manifests(
        records: Sequence[SampleRecord],
        assignments: Dict[str, List[str]],
        duplicate_report: Dict[str, Dict[str, object]],
) -> Dict[str, List[dict]]:
    by_id = {record.sample_id: record for record in records}
    normalized_to_ids = {key: set(value["member_ids"]) for key, value in duplicate_report.items()}

    manifests: Dict[str, List[dict]] = {}
    for split_name, sample_ids in assignments.items():
        rows: List[dict] = []
        for sample_id in sample_ids:
            record = by_id[sample_id]
            dup_group_size = 1
            for ids in normalized_to_ids.values():
                if sample_id in ids:
                    dup_group_size = len(ids)
                    break
            rows.append(
                {
                    "id": record.sample_id,
                    "source_root": record.source_root,
                    "source_id": record.source_stem,
                    "problem_path": str(record.problem_path),
                    "solution_path": str(record.solution_path),
                    "test_path": str(record.test_path) if record.test_path is not None else None,
                    "has_test": record.has_test,
                    "duplicate_group_size": dup_group_size,
                }
            )
        manifests[split_name] = rows
    return manifests



def count_lines(path: Path, encoding: str) -> int:
    return len(path.read_text(encoding=encoding).splitlines())



def build_summary(
        records: Sequence[SampleRecord],
        assignments: Dict[str, List[str]],
        missing: Dict[str, List[str]],
        duplicate_report: Dict[str, Dict[str, object]],
        encoding: str,
        target_counts: Dict[str, int],
        action: str,
        dry_run: bool,
        seed: int,
        roots: Sequence[Path],
) -> dict:
    by_id = {record.sample_id: record for record in records}
    split_summary = {}

    for split_name, sample_ids in assignments.items():
        problem_lines = 0
        solution_lines = 0
        test_lines = 0
        samples_with_tests = 0
        for sample_id in sample_ids:
            record = by_id[sample_id]
            problem_lines += count_lines(record.problem_path, encoding)
            solution_lines += count_lines(record.solution_path, encoding)
            if record.test_path is not None:
                test_lines += count_lines(record.test_path, encoding)
                samples_with_tests += 1
        split_summary[split_name] = {
            "count": len(sample_ids),
            "target_count": target_counts[split_name],
            "samples_with_tests": samples_with_tests,
            "problem_lines": problem_lines,
            "solution_lines": solution_lines,
            "test_lines": test_lines,
            "sample_ids": sample_ids,
        }

    test_eligible_ids = sorted(record.sample_id for record in records if record.has_test)

    return {
        "input_roots": [str(root) for root in roots],
        "total_paired_samples": len(records),
        "test_eligible_samples": len(test_eligible_ids),
        "test_eligible_sample_ids": test_eligible_ids,
        "missing": missing,
        "duplicate_problem_groups": {
            "count": len(duplicate_report),
            "extra_duplicate_samples": sum(max(0, v["size"] - 1) for v in duplicate_report.values()),
            "groups": duplicate_report,
        },
        "split_summary": split_summary,
        "seed": seed,
        "action": action,
        "dry_run": dry_run,
    }



def main() -> int:
    args = parse_args()

    roots = flatten_roots(args.roots)
    output_dir = Path(args.output_dir)

    if not roots:
        print("ERROR: at least one root directory must be provided.", file=sys.stderr)
        return 2

    try:
        ensure_output_dir(output_dir, overwrite=args.overwrite)
    except Exception as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 2

    try:
        records, missing = collect_records(
            roots=roots,
            problems_subdir=args.problems_dir,
            solutions_subdir=args.solutions_dir,
            tests_subdir=args.tests_dir,
            problem_suffix=args.problem_suffix,
            solution_suffix=args.solution_suffix,
            test_suffix=args.test_suffix,
            encoding=args.encoding,
        )
    except Exception as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 2

    if not records:
        print("ERROR: no fully paired problem/solution samples were found.", file=sys.stderr)
        return 2

    test_eligible_total = sum(1 for record in records if record.has_test)

    try:
        target_counts = derive_target_counts(
            total=len(records),
            test_eligible_total=test_eligible_total,
            train_ratio=args.train_ratio,
            val_ratio=args.val_ratio,
            test_ratio=args.test_ratio,
            split_counts=args.split_counts,
        )
    except Exception as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 2

    groups, duplicate_report = build_groups(records, group_duplicates=args.group_duplicates, seed=args.seed)
    assignments = assign_groups_to_splits(groups, target_counts=target_counts, seed=args.seed)
    manifests = build_manifests(records, assignments, duplicate_report)
    summary = build_summary(
        records=records,
        assignments=assignments,
        missing=missing,
        duplicate_report=duplicate_report,
        encoding=args.encoding,
        target_counts=target_counts,
        action=args.action,
        dry_run=args.dry_run,
        seed=args.seed,
        roots=roots,
    )

    if not args.dry_run:
        by_id = {record.sample_id: record for record in records}
        for split_name, sample_ids in assignments.items():
            for sample_id in sample_ids:
                record = by_id[sample_id]
                split_dir = output_dir / split_name
                materialize_file(record.problem_path, split_dir / "problems" / record.problem_path.name, action=args.action)
                materialize_file(record.solution_path, split_dir / "solutions" / record.solution_path.name, action=args.action)
                if record.test_path is not None:
                    materialize_file(record.test_path, split_dir / "tests" / record.test_path.name, action=args.action)

    write_json(output_dir / "split_summary.json", summary)

    for split_name, rows in manifests.items():
        write_jsonl(output_dir / f"{split_name}_manifest.jsonl", rows)

    console_counts = {name: len(ids) for name, ids in assignments.items()}
    print("Input roots:", ", ".join(str(root) for root in roots))
    print("Paired samples:", len(records))
    print("Test-eligible samples:", test_eligible_total)
    print("Target split counts:", target_counts)
    print("Actual split counts:", console_counts)
    print("Duplicate problem groups:", len(duplicate_report))

    if missing["roots_without_tests_dir"]:
        print("WARNING: roots without tests directory detected:", len(missing["roots_without_tests_dir"]))
        for root in missing["roots_without_tests_dir"][:10]:
            print(f"  no tests dir: {root}")
        if len(missing["roots_without_tests_dir"]) > 10:
            print(f"  ... (+{len(missing['roots_without_tests_dir']) - 10} more)")

    if missing["unpaired_any"]:
        print("WARNING: unpaired sample ids detected:", len(missing["unpaired_any"]))
        for key in ("missing_problem", "missing_solution", "missing_test"):
            if missing[key]:
                preview = ", ".join(missing[key][:10])
                more = "" if len(missing[key]) <= 10 else f" ... (+{len(missing[key]) - 10} more)"
                print(f"  {key}: {preview}{more}")

    if duplicate_report:
        print("WARNING: normalized duplicate problems were found and grouped into the same split.")
        preview_items = list(duplicate_report.values())[:5]
        for item in preview_items:
            print(
                f"  duplicate group size={item['size']} test_eligible={item['test_eligible']}: "
                f"{', '.join(item['member_ids'])}"
            )

    if args.dry_run:
        print(f"Dry run complete. Summary written to: {output_dir / 'split_summary.json'}")
    else:
        print(f"Split materialization complete in: {output_dir}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
