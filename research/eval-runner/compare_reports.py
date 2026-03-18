#!/usr/bin/env python3
"""Compare eval-runner run_report.json files across benchmarks and models."""

from __future__ import annotations

import argparse
import csv
import json
from collections import Counter
from pathlib import Path
from typing import Any


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Discover and compare eval-runner run_report.json files.\n\n"
            "Expected layout: <reports-dir>/<benchmark>/<model>/run_report.json\n"
            "If a report path does not match this shape, the script falls back to a\n"
            "best-effort key based on the full relative path. Use --benchmark to\n"
            "filter benchmark groups after discovery."
        ),
        formatter_class=argparse.RawTextHelpFormatter,
    )
    parser.add_argument(
        "--reports-dir",
        default="./out",
        help="Root directory to scan for run_report.json files (default: ./out).",
    )
    parser.add_argument(
        "--out",
        default=None,
        help="Optional output path for combined comparison CSV.",
    )
    parser.add_argument(
        "--benchmark",
        action="append",
        default=None,
        help="Benchmark filter (repeatable). Example: --benchmark humaneval-rust",
    )
    parser.add_argument(
        "--per-problem",
        action="store_true",
        help="Print per-problem breakdown tables for each (benchmark, model).",
    )
    return parser.parse_args()


def safe_float(value: Any, default: float = 0.0) -> float:
    try:
        if value is None:
            return default
        return float(value)
    except (TypeError, ValueError):
        return default


def safe_int(value: Any, default: int = 0) -> int:
    try:
        if value is None:
            return default
        return int(value)
    except (TypeError, ValueError):
        return default


def top_counter_items(counter: Counter[str], limit: int = 5) -> list[tuple[str, int]]:
    return counter.most_common(limit)


def format_top_codes(counter: Counter[str], limit: int = 5) -> str:
    items = top_counter_items(counter, limit)
    if not items:
        return "-"
    return ", ".join(f"{code}:{count}" for code, count in items)


def add_codes(counter: Counter[str], value: Any) -> None:
    if value is None:
        return

    if isinstance(value, dict):
        for code, count in value.items():
            counter[str(code)] += safe_int(count, 1)
        return

    if isinstance(value, list):
        for item in value:
            if isinstance(item, str):
                counter[item] += 1
            elif isinstance(item, dict):
                code = item.get("code") or item.get("name") or item.get("id")
                count = item.get("count", 1)
                if code is not None:
                    counter[str(code)] += safe_int(count, 1)
            elif isinstance(item, (tuple, list)) and len(item) == 2:
                code, count = item
                counter[str(code)] += safe_int(count, 1)


def infer_benchmark_model(root: Path, report_path: Path) -> tuple[str, str, bool]:
    rel = report_path.relative_to(root)
    parts = rel.parts

    # Expected: <benchmark>/<model>/run_report.json
    if len(parts) >= 3 and parts[-1] == "run_report.json":
        return parts[-3], parts[-2], False

    relative_key = rel.as_posix().removesuffix("/run_report.json")
    if not relative_key:
        relative_key = rel.as_posix()
    return "(unparsed-path)", relative_key, True


def load_report(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def aggregate_reports(
    reports_dir: Path, benchmark_filter: set[str] | None
) -> dict[tuple[str, str], dict[str, Any]]:
    grouped: dict[tuple[str, str], dict[str, Any]] = {}

    for report_path in sorted(reports_dir.rglob("run_report.json")):
        benchmark, model, was_fallback = infer_benchmark_model(reports_dir, report_path)
        payload = load_report(report_path)

        meta = payload.get("meta", {}) if isinstance(payload.get("meta"), dict) else {}
        meta_model = meta.get("model")
        if was_fallback and isinstance(meta_model, str) and meta_model.strip():
            model = meta_model.strip()

        if benchmark_filter and benchmark not in benchmark_filter:
            continue

        key = (benchmark, model)
        if key not in grouped:
            grouped[key] = {
                "benchmark": benchmark,
                "model": model,
                "problems": 0,
                "sum_build_rate": 0.0,
                "sum_test_rate": 0.0,
                "sum_overall_ok_rate": 0.0,
                "sum_clippy_warnings": 0.0,
                "sum_clippy_errors": 0.0,
                "pass_at_1_numer": 0,
                "pass_at_k_numer": 0,
                "top_build_codes": Counter(),
                "top_clippy_codes": Counter(),
                "meta_k_values": set(),
                "meta_started_min": None,
                "meta_started_max": None,
                "report_paths": [],
                "per_problem": [],
            }

        group = grouped[key]
        group["report_paths"].append(str(report_path))

        k_value = meta.get("k")
        if k_value is not None:
            group["meta_k_values"].add(str(k_value))

        started = meta.get("started_unix_ms")
        started_int = safe_int(started, -1)
        if started_int >= 0:
            if group["meta_started_min"] is None or started_int < group["meta_started_min"]:
                group["meta_started_min"] = started_int
            if group["meta_started_max"] is None or started_int > group["meta_started_max"]:
                group["meta_started_max"] = started_int

        problems = payload.get("problems", [])
        if not isinstance(problems, list):
            continue

        for problem in problems:
            if not isinstance(problem, dict):
                continue

            stats = problem.get("stats", {})
            if not isinstance(stats, dict):
                stats = {}

            build_rate = safe_float(stats.get("build_success_rate"))
            test_rate = safe_float(stats.get("test_success_rate"))
            overall_ok_rate = safe_float(stats.get("overall_ok_rate"))
            avg_clippy_warnings = safe_float(stats.get("avg_clippy_warnings"))
            avg_clippy_errors = safe_float(stats.get("avg_clippy_errors"))

            group["problems"] += 1
            group["sum_build_rate"] += build_rate
            group["sum_test_rate"] += test_rate
            group["sum_overall_ok_rate"] += overall_ok_rate
            group["sum_clippy_warnings"] += avg_clippy_warnings
            group["sum_clippy_errors"] += avg_clippy_errors

            if overall_ok_rate > 0.0:
                group["pass_at_1_numer"] += 1
            if overall_ok_rate == 1.0:
                group["pass_at_k_numer"] += 1

            add_codes(group["top_build_codes"], stats.get("top_build_codes"))
            add_codes(group["top_clippy_codes"], stats.get("top_clippy_codes"))

            problem_id = (
                problem.get("problem_id")
                or problem.get("id")
                or problem.get("name")
                or f"problem-{group['problems']}"
            )
            group["per_problem"].append(
                {
                    "problem_id": str(problem_id),
                    "build_rate": build_rate,
                    "test_rate": test_rate,
                    "overall_ok_rate": overall_ok_rate,
                }
            )

    return grouped


def summarize(group: dict[str, Any]) -> dict[str, Any]:
    total = group["problems"]
    denom = float(total) if total > 0 else 1.0
    return {
        "benchmark": group["benchmark"],
        "model": group["model"],
        "problems": total,
        "pass_at_1": group["pass_at_1_numer"] / denom if total else 0.0,
        "pass_at_k": group["pass_at_k_numer"] / denom if total else 0.0,
        "avg_build_rate": group["sum_build_rate"] / denom if total else 0.0,
        "avg_test_rate": group["sum_test_rate"] / denom if total else 0.0,
        "avg_clippy_warnings": group["sum_clippy_warnings"] / denom if total else 0.0,
        "avg_clippy_errors": group["sum_clippy_errors"] / denom if total else 0.0,
        "top_build_codes": top_counter_items(group["top_build_codes"], 5),
        "top_clippy_codes": top_counter_items(group["top_clippy_codes"], 5),
        "top_build_codes_str": format_top_codes(group["top_build_codes"], 5),
        "top_clippy_codes_str": format_top_codes(group["top_clippy_codes"], 5),
    }


def fmt_pct(value: float) -> str:
    return f"{value * 100:6.1f}%"


def fmt_num(value: float, width: int = 7, decimals: int = 2) -> str:
    return f"{value:{width}.{decimals}f}"


def print_main_table(by_benchmark: dict[str, list[dict[str, Any]]]) -> None:
    model_w = 25
    problems_w = 8
    pct_w = 7
    clippy_w = 10

    header = (
        f"{'Model'.ljust(model_w)} | "
        f"{'Problems'.rjust(problems_w)} | "
        f"{'pass@1'.rjust(pct_w)} | "
        f"{'pass@k'.rjust(pct_w)} | "
        f"{'Build%'.rjust(pct_w)} | "
        f"{'Test%'.rjust(pct_w)} | "
        f"{'ClippyWarn'.rjust(clippy_w)} | "
        f"{'ClippyErr'.rjust(clippy_w)}"
    )
    separator = "-" * len(header)

    for benchmark in sorted(by_benchmark):
        print(f"\n=== Benchmark: {benchmark} ===")
        print(header)
        print(separator)
        rows = sorted(
            by_benchmark[benchmark],
            key=lambda row: (-row["pass_at_1"], -row["pass_at_k"], row["model"]),
        )
        for row in rows:
            print(
                f"{row['model'][:model_w].ljust(model_w)} | "
                f"{str(row['problems']).rjust(problems_w)} | "
                f"{fmt_pct(row['pass_at_1']).rjust(pct_w)} | "
                f"{fmt_pct(row['pass_at_k']).rjust(pct_w)} | "
                f"{fmt_pct(row['avg_build_rate']).rjust(pct_w)} | "
                f"{fmt_pct(row['avg_test_rate']).rjust(pct_w)} | "
                f"{fmt_num(row['avg_clippy_warnings'], clippy_w, 2).rjust(clippy_w)} | "
                f"{fmt_num(row['avg_clippy_errors'], clippy_w, 2).rjust(clippy_w)}"
            )


def print_top_codes(by_benchmark: dict[str, list[dict[str, Any]]]) -> None:
    for benchmark in sorted(by_benchmark):
        print(f"\nTop diagnostic codes for benchmark: {benchmark}")
        rows = sorted(
            by_benchmark[benchmark],
            key=lambda row: (-row["pass_at_1"], -row["pass_at_k"], row["model"]),
        )
        for row in rows:
            print(f"- {row['model']}")
            print(f"  build:  {row['top_build_codes_str']}")
            print(f"  clippy: {row['top_clippy_codes_str']}")


def print_per_problem(grouped: dict[tuple[str, str], dict[str, Any]]) -> None:
    model_w = 25
    problem_w = 48
    pct_w = 7
    header = (
        f"{'Problem'.ljust(problem_w)} | "
        f"{'Build%'.rjust(pct_w)} | "
        f"{'Test%'.rjust(pct_w)} | "
        f"{'Overall%'.rjust(pct_w)}"
    )
    separator = "-" * len(header)

    for key in sorted(grouped):
        group = grouped[key]
        benchmark = group["benchmark"]
        model = group["model"]
        print(f"\nPer-problem: {benchmark} / {model[:model_w]}")
        print(header)
        print(separator)
        for row in sorted(group["per_problem"], key=lambda item: item["problem_id"]):
            problem_id = row["problem_id"][:problem_w]
            print(
                f"{problem_id.ljust(problem_w)} | "
                f"{fmt_pct(row['build_rate']).rjust(pct_w)} | "
                f"{fmt_pct(row['test_rate']).rjust(pct_w)} | "
                f"{fmt_pct(row['overall_ok_rate']).rjust(pct_w)}"
            )


def write_csv(path: Path, rows: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    columns = [
        "benchmark",
        "model",
        "problems",
        "pass_at_1",
        "pass_at_k",
        "avg_build_rate",
        "avg_test_rate",
        "avg_clippy_warnings",
        "avg_clippy_errors",
        "top_build_codes",
        "top_clippy_codes",
    ]
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=columns)
        writer.writeheader()
        for row in rows:
            writer.writerow(
                {
                    "benchmark": row["benchmark"],
                    "model": row["model"],
                    "problems": row["problems"],
                    "pass_at_1": f"{row['pass_at_1']:.6f}",
                    "pass_at_k": f"{row['pass_at_k']:.6f}",
                    "avg_build_rate": f"{row['avg_build_rate']:.6f}",
                    "avg_test_rate": f"{row['avg_test_rate']:.6f}",
                    "avg_clippy_warnings": f"{row['avg_clippy_warnings']:.6f}",
                    "avg_clippy_errors": f"{row['avg_clippy_errors']:.6f}",
                    "top_build_codes": row["top_build_codes_str"],
                    "top_clippy_codes": row["top_clippy_codes_str"],
                }
            )


def main() -> int:
    args = parse_args()
    reports_dir = Path(args.reports_dir).resolve()

    if not reports_dir.exists():
        raise SystemExit(f"Reports directory does not exist: {reports_dir}")

    benchmark_filter = set(args.benchmark) if args.benchmark else None
    grouped = aggregate_reports(reports_dir, benchmark_filter)

    if not grouped:
        print("No run_report.json files matched the provided filters.")
        return 0

    rows = [summarize(group) for group in grouped.values()]

    by_benchmark: dict[str, list[dict[str, Any]]] = {}
    for row in rows:
        by_benchmark.setdefault(row["benchmark"], []).append(row)

    print_main_table(by_benchmark)
    print_top_codes(by_benchmark)

    if args.per_problem:
        print_per_problem(grouped)

    if args.out:
        out_path = Path(args.out).resolve()
        write_csv(out_path, sorted(rows, key=lambda row: (row["benchmark"], row["model"])))
        print(f"\nWrote combined CSV to: {out_path}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
