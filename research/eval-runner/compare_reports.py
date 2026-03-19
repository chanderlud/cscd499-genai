#!/usr/bin/env python3
"""Compare eval-runner run_report.json files across benchmarks and models."""

from __future__ import annotations

import argparse
import csv
import json
import re
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
    parser.add_argument(
        "--compile-errors",
        action="store_true",
        help="Print detailed compile-error analysis per benchmark and model.",
    )
    parser.add_argument(
        "--charts-dir",
        default=None,
        help=(
            "Optional output directory for PNG chart files. "
            "When omitted, no charts are produced."
        ),
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


def normalize_error_message(msg: str) -> str:
    """Normalize a compiler error message for clustering.

    Replaces backtick-quoted content with <X>, strips error code brackets,
    lowercases, and truncates to 120 chars.
    """
    text = re.sub(r"`[^`]*`", "<X>", msg)
    text = re.sub(r"\[E\d+\]", "", text)
    text = text.lower().strip()
    if len(text) > 120:
        text = text[:120]
    return text


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
                "build_error_codes": Counter(),
                "build_error_messages": Counter(),
                "no_eval_count": 0,
                "build_fail_all_count": 0,
                "total_attempts": 0,
                "total_build_fail_attempts": 0,
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

            attempts = problem.get("attempts", [])
            if not isinstance(attempts, list):
                attempts = []
            problem_build_fail_count = 0
            for attempt in attempts:
                if not isinstance(attempt, dict):
                    continue
                group["total_attempts"] += 1
                eval_data = attempt.get("eval")
                if eval_data is None:
                    group["no_eval_count"] += 1
                    problem_build_fail_count += 1
                    continue
                if not isinstance(eval_data, dict):
                    continue
                build = eval_data.get("build")
                if not isinstance(build, dict):
                    continue
                if not build.get("ok", False):
                    group["total_build_fail_attempts"] += 1
                    problem_build_fail_count += 1
                    diagnostics = build.get("diagnostics")
                    if isinstance(diagnostics, dict):
                        by_code = diagnostics.get("by_code")
                        if isinstance(by_code, dict):
                            for code, count in by_code.items():
                                group["build_error_codes"][str(code)] += safe_int(count, 1)
                        items = diagnostics.get("items")
                        if isinstance(items, list):
                            for item in items:
                                if isinstance(item, dict) and item.get("level") == "error":
                                    raw_msg = item.get("message", "")
                                    if raw_msg:
                                        normalized = normalize_error_message(raw_msg)
                                        if normalized:
                                            group["build_error_messages"][normalized] += 1
            if len(attempts) > 0 and problem_build_fail_count == len(attempts):
                group["build_fail_all_count"] += 1

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
    total_attempts = group["total_attempts"]
    att_denom = float(total_attempts) if total_attempts > 0 else 1.0
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
        "build_fail_rate": group["total_build_fail_attempts"] / att_denom if total_attempts else 0.0,
        "no_eval_rate": group["no_eval_count"] / att_denom if total_attempts else 0.0,
        "build_fail_all_pct": group["build_fail_all_count"] / denom if total else 0.0,
        "top_build_error_codes": top_counter_items(group["build_error_codes"], 10),
        "top_build_error_messages": top_counter_items(group["build_error_messages"], 10),
        "top_build_error_codes_str": format_top_codes(group["build_error_codes"], 10),
        "top_build_error_messages_str": format_top_codes(group["build_error_messages"], 10),
    }


def fmt_pct(value: float) -> str:
    return f"{value * 100:6.1f}%"


def fmt_num(value: float, width: int = 7, decimals: int = 2) -> str:
    return f"{value:{width}.{decimals}f}"


def print_main_table(by_benchmark: dict[str, list[dict[str, Any]]]) -> None:
    model_w = 25
    problems_w = 8
    pct_w = 7
    fail_w = 10
    clippy_w = 10

    header = (
        f"{'Model'.ljust(model_w)} | "
        f"{'Problems'.rjust(problems_w)} | "
        f"{'pass@1'.rjust(pct_w)} | "
        f"{'pass@k'.rjust(pct_w)} | "
        f"{'Build%'.rjust(pct_w)} | "
        f"{'Test%'.rjust(pct_w)} | "
        f"{'BuildFail%'.rjust(fail_w)} | "
        f"{'0/k%'.rjust(pct_w)} | "
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
                f"{fmt_pct(row['build_fail_rate']).rjust(fail_w)} | "
                f"{fmt_pct(row['build_fail_all_pct']).rjust(pct_w)} | "
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


def print_compile_error_analysis(by_benchmark: dict[str, list[dict[str, Any]]]) -> None:
    for benchmark in sorted(by_benchmark):
        print(f"\n=== Compile Error Analysis: {benchmark} ===")
        rows = sorted(
            by_benchmark[benchmark],
            key=lambda row: (-row["build_fail_rate"], row["model"]),
        )
        for row in rows:
            print(f"\nModel: {row['model']}")
            print(f"  Build fail rate (attempt-level): {fmt_pct(row['build_fail_rate'])}")
            print(f"  Problems where 0/k compiled:     {fmt_pct(row['build_fail_all_pct'])}")
            print(f"  No-eval attempts (gen failure):  {fmt_pct(row['no_eval_rate'])}")

            if row["top_build_error_codes"]:
                print("\n  Top error codes (build failures only):")
                code_w = max(len(code) for code, _ in row["top_build_error_codes"])
                for code, count in row["top_build_error_codes"]:
                    print(f"    {code.ljust(code_w)} : {count}")

            if row["top_build_error_messages"]:
                print("\n  Top error messages (normalized, build failures only):")
                msg_w = min(max(len(m) for m, _ in row["top_build_error_messages"]), 60)
                for msg, count in row["top_build_error_messages"]:
                    display = msg[:60]
                    print(f"    {display.ljust(msg_w)} : {count}")


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
        "build_fail_all_pct",
        "no_eval_rate",
        "top_build_error_codes",
        "top_build_error_messages",
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
                    "build_fail_all_pct": f"{row['build_fail_all_pct']:.6f}",
                    "no_eval_rate": f"{row['no_eval_rate']:.6f}",
                    "top_build_error_codes": row["top_build_error_codes_str"],
                    "top_build_error_messages": row["top_build_error_messages_str"],
                }
            )


def _safe_filename_component(name: str) -> str:
    return re.sub(r"[^A-Za-z0-9._-]+", "_", name).strip("_") or "unknown"


def _chart_model_overview(
    by_benchmark: dict[str, list[dict[str, Any]]], charts_dir: Path, plt: Any
) -> None:
    metric_specs = [
        ("pass@1", "pass_at_1"),
        ("pass@k", "pass_at_k"),
        ("avg_build_rate", "avg_build_rate"),
        ("avg_test_rate", "avg_test_rate"),
    ]
    for benchmark, rows in sorted(by_benchmark.items()):
        if not rows:
            continue
        ordered = sorted(rows, key=lambda row: (-row["pass_at_1"], row["model"]))
        models = [row["model"] for row in ordered]
        x = list(range(len(models)))
        width = 0.18
        offsets = [-1.5 * width, -0.5 * width, 0.5 * width, 1.5 * width]

        fig, ax = plt.subplots(figsize=(max(10, len(models) * 1.4), 6))
        for offset, (label, key) in zip(offsets, metric_specs):
            values = [row[key] * 100.0 for row in ordered]
            bars = ax.bar([i + offset for i in x], values, width=width, label=label)
            for bar, value in zip(bars, values):
                ax.text(
                    bar.get_x() + bar.get_width() / 2.0,
                    bar.get_height() + 0.7,
                    f"{value:.1f}%",
                    ha="center",
                    va="bottom",
                    fontsize=8,
                )

        ax.set_xticks(x)
        ax.set_xticklabels(models, rotation=30, ha="right")
        ax.set_ylim(0.0, 105.0)
        ax.set_ylabel("Percent")
        ax.set_title(f"Model Performance Overview: {benchmark}")
        ax.legend(loc="upper right")
        ax.grid(axis="y", alpha=0.3, linestyle="--")
        fig.tight_layout()
        out_path = charts_dir / f"{_safe_filename_component(benchmark)}_overview.png"
        fig.savefig(out_path, bbox_inches="tight")
        plt.close(fig)


def _chart_quality_metrics(
    by_benchmark: dict[str, list[dict[str, Any]]], charts_dir: Path, plt: Any
) -> None:
    for benchmark, rows in sorted(by_benchmark.items()):
        if not rows:
            continue
        ordered = sorted(rows, key=lambda row: (-row["pass_at_1"], row["model"]))
        models = [row["model"] for row in ordered]
        x = list(range(len(models)))
        warnings = [row["avg_clippy_warnings"] for row in ordered]
        errors = [row["avg_clippy_errors"] for row in ordered]
        fail_rate = [row["build_fail_rate"] * 100.0 for row in ordered]

        fig, ax1 = plt.subplots(figsize=(max(10, len(models) * 1.4), 6))
        ax1.bar(x, warnings, label="avg_clippy_warnings", color="#f1c40f")
        ax1.bar(x, errors, bottom=warnings, label="avg_clippy_errors", color="#e67e22")
        ax1.set_ylabel("Avg clippy issues / problem")
        ax1.set_xticks(x)
        ax1.set_xticklabels(models, rotation=30, ha="right")
        ax1.grid(axis="y", alpha=0.25, linestyle="--")

        ax2 = ax1.twinx()
        ax2.plot(x, fail_rate, color="#c0392b", marker="o", linewidth=2, label="build_fail_rate")
        ax2.set_ylim(0.0, 100.0)
        ax2.set_ylabel("Build fail rate (%)")

        handles1, labels1 = ax1.get_legend_handles_labels()
        handles2, labels2 = ax2.get_legend_handles_labels()
        ax1.legend(handles1 + handles2, labels1 + labels2, loc="upper right")
        ax1.set_title(f"Build & Clippy Quality: {benchmark}")
        fig.tight_layout()
        out_path = charts_dir / f"{_safe_filename_component(benchmark)}_quality.png"
        fig.savefig(out_path, bbox_inches="tight")
        plt.close(fig)


def _chart_error_codes(
    by_benchmark: dict[str, list[dict[str, Any]]], charts_dir: Path, plt: Any
) -> None:
    for benchmark, rows in sorted(by_benchmark.items()):
        if not rows:
            continue
        ordered = sorted(rows, key=lambda row: (-row["pass_at_1"], row["model"]))
        model_count = len(ordered)
        fig, axes = plt.subplots(
            nrows=model_count,
            ncols=1,
            figsize=(10, max(3.0 * model_count, 4.0)),
            squeeze=False,
        )
        for idx, row in enumerate(ordered):
            ax = axes[idx][0]
            codes = row.get("top_build_error_codes", [])
            if not codes:
                ax.text(0.5, 0.5, "No build error codes", ha="center", va="center", transform=ax.transAxes)
                ax.set_yticks([])
                ax.set_xticks([])
                ax.set_title(row["model"])
                continue
            sorted_codes = sorted(codes, key=lambda item: item[1], reverse=True)
            labels = [code for code, _ in sorted_codes]
            counts = [count for _, count in sorted_codes]
            y = list(range(len(labels)))
            ax.barh(y, counts, color="#3498db")
            ax.set_yticks(y)
            ax.set_yticklabels(labels)
            ax.invert_yaxis()
            ax.set_xlabel("Count")
            ax.set_title(row["model"])
            for yi, count in zip(y, counts):
                ax.text(count + 0.05, yi, str(count), va="center", fontsize=8)
            ax.grid(axis="x", alpha=0.25, linestyle="--")
        fig.suptitle(f"Top Build Error Codes: {benchmark}", y=1.01)
        fig.tight_layout()
        out_path = charts_dir / f"{_safe_filename_component(benchmark)}_error_codes.png"
        fig.savefig(out_path, bbox_inches="tight")
        plt.close(fig)


def _chart_per_problem_heatmap(
    by_benchmark: dict[str, list[dict[str, Any]]],
    grouped: dict[tuple[str, str], dict[str, Any]],
    charts_dir: Path,
    plt: Any,
) -> None:
    for benchmark, rows in sorted(by_benchmark.items()):
        if not rows:
            continue
        ordered = sorted(rows, key=lambda row: (-row["pass_at_1"], row["model"]))
        models = [row["model"] for row in ordered]
        problem_ids: set[str] = set()
        per_model_maps: dict[str, dict[str, float]] = {}

        for model in models:
            group = grouped.get((benchmark, model), {})
            per_problem = group.get("per_problem", [])
            model_map: dict[str, float] = {}
            if isinstance(per_problem, list):
                for item in per_problem:
                    if not isinstance(item, dict):
                        continue
                    pid = str(item.get("problem_id", ""))
                    rate = safe_float(item.get("overall_ok_rate"))
                    model_map[pid] = rate
                    problem_ids.add(pid)
            per_model_maps[model] = model_map

        sorted_problem_ids = sorted(problem_ids)
        if not sorted_problem_ids:
            continue
        matrix = [
            [per_model_maps[model].get(pid, 0.0) for pid in sorted_problem_ids]
            for model in models
        ]

        fig, ax = plt.subplots(
            figsize=(max(10, len(sorted_problem_ids) * 0.4), max(4, len(models) * 0.6))
        )
        heat = ax.imshow(matrix, aspect="auto", vmin=0.0, vmax=1.0, cmap="RdYlGn")
        ax.set_yticks(list(range(len(models))))
        ax.set_yticklabels(models)
        ax.set_xticks(list(range(len(sorted_problem_ids))))
        ax.set_xticklabels(sorted_problem_ids, rotation=90, fontsize=8)
        ax.set_title(f"Per-Problem Overall OK Rate Heatmap: {benchmark}")
        ax.set_xlabel("Problem ID")
        ax.set_ylabel("Model")
        cbar = fig.colorbar(heat, ax=ax)
        cbar.set_label("overall_ok_rate")
        fig.tight_layout()
        out_path = charts_dir / f"{_safe_filename_component(benchmark)}_heatmap.png"
        fig.savefig(out_path, bbox_inches="tight")
        plt.close(fig)


def _chart_radar(
    by_benchmark: dict[str, list[dict[str, Any]]], charts_dir: Path, plt: Any
) -> None:
    try:
        import numpy as np
    except ImportError:
        print("Skipping radar charts because numpy is unavailable.")
        return

    axis_specs = [
        ("pass@1", "pass_at_1"),
        ("pass@k", "pass_at_k"),
        ("avg_build_rate", "avg_build_rate"),
        ("avg_test_rate", "avg_test_rate"),
        ("1-build_fail_rate", "build_fail_rate"),
        ("1-build_fail_all_pct", "build_fail_all_pct"),
    ]
    labels = [label for label, _ in axis_specs]
    axis_count = len(axis_specs)
    angles = np.linspace(0, 2 * np.pi, axis_count, endpoint=False)
    angles = np.append(angles, angles[0])

    for benchmark, rows in sorted(by_benchmark.items()):
        if not rows:
            continue
        ordered = sorted(rows, key=lambda row: (-row["pass_at_1"], row["model"]))
        fig, ax = plt.subplots(figsize=(8, 8), subplot_kw={"projection": "polar"})
        for row in ordered:
            values = []
            for _, key in axis_specs:
                value = safe_float(row.get(key))
                if key in {"build_fail_rate", "build_fail_all_pct"}:
                    value = 1.0 - value
                values.append(max(0.0, min(1.0, value)))
            values = np.append(values, values[0])
            ax.plot(angles, values, linewidth=2, label=row["model"])
            ax.fill(angles, values, alpha=0.08)

        ax.set_xticks(angles[:-1])
        ax.set_xticklabels(labels)
        ax.set_ylim(0.0, 1.0)
        ax.set_yticks([0.2, 0.4, 0.6, 0.8, 1.0])
        ax.set_title(f"Radar Profile: {benchmark}", pad=24)
        ax.legend(loc="upper right", bbox_to_anchor=(1.35, 1.12))
        fig.tight_layout()
        out_path = charts_dir / f"{_safe_filename_component(benchmark)}_radar.png"
        fig.savefig(out_path, bbox_inches="tight")
        plt.close(fig)


def generate_charts(
    by_benchmark: dict[str, list[dict[str, Any]]],
    grouped: dict[tuple[str, str], dict[str, Any]],
    charts_dir: Path,
) -> None:
    try:
        import matplotlib.pyplot as plt
    except ImportError:
        print(
            "Chart generation requested, but matplotlib is not installed. "
            "Install it with: pip install matplotlib"
        )
        return

    _chart_model_overview(by_benchmark, charts_dir, plt)
    _chart_quality_metrics(by_benchmark, charts_dir, plt)
    _chart_error_codes(by_benchmark, charts_dir, plt)
    _chart_per_problem_heatmap(by_benchmark, grouped, charts_dir, plt)
    _chart_radar(by_benchmark, charts_dir, plt)


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

    if args.compile_errors:
        print_compile_error_analysis(by_benchmark)

    if args.per_problem:
        print_per_problem(grouped)

    if args.charts_dir:
        charts_dir = Path(args.charts_dir).resolve()
        charts_dir.mkdir(parents=True, exist_ok=True)
        generate_charts(by_benchmark, grouped, charts_dir)
        print(f"\nWrote charts to: {charts_dir}")

    if args.out:
        out_path = Path(args.out).resolve()
        write_csv(out_path, sorted(rows, key=lambda row: (row["benchmark"], row["model"])))
        print(f"\nWrote combined CSV to: {out_path}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
