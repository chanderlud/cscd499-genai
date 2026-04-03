#!/usr/bin/env python3
"""Load and reshape local JSONL files into prompt/completion JSONL.

Each output row has the form:
{
  "prompt": [
    {"role": "system", "content": "..."},
    {"role": "user", "content": "..."}
  ],
  "completion": [
    {"role": "assistant", "content": "..."}
  ]
}

Input:
- A local directory containing one or more .jsonl files
- Each line should be a JSON object

Supports:
- category discovery
- include/exclude filters
- per-category balancing via min/max/mean/median/target
- per-category weights
- optional oversampling
- optional train/val split output
- optional stratified splitting by category
"""

from __future__ import annotations

import argparse
import json
import math
import random
from collections import Counter, defaultdict
from pathlib import Path
from statistics import median
from typing import Any, Iterable

AUTO_CATEGORY_FIELDS = ("category", "task", "task_type", "label", "subcategory")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)

    parser.add_argument("--input-dir", required=True, help="Directory containing .jsonl files")
    parser.add_argument("--pattern", default="*.jsonl", help="Glob pattern for input files")
    parser.add_argument("--recursive", action="store_true", help="Recursively search input-dir")

    parser.add_argument("--out", help="Single output JSONL path")
    parser.add_argument("--train-out", help="Train split output JSONL path")
    parser.add_argument("--val-out", help="Validation split output JSONL path")
    parser.add_argument("--val-ratio", type=float, default=None, help="Validation fraction, e.g. 0.1")
    parser.add_argument("--val-count", type=int, default=None, help="Exact number of validation rows")

    parser.add_argument("--max-rows", type=int, default=None)
    parser.add_argument("--skip-missing-assistant", action="store_true")
    parser.add_argument("--category-field", default="auto")
    parser.add_argument("--list-categories", action="store_true")
    parser.add_argument("--include-categories", default=None)
    parser.add_argument("--exclude-categories", default=None)
    parser.add_argument(
        "--balance-mode",
        choices=("none", "min", "max", "mean", "median", "target"),
        default="none",
    )
    parser.add_argument("--target-per-category", type=int, default=None)
    parser.add_argument("--category-weights", default=None)
    parser.add_argument("--allow-oversample", action="store_true")
    parser.add_argument("--shuffle", action="store_true")
    parser.add_argument("--seed", type=int, default=42)

    args = parser.parse_args()

    using_split_outputs = bool(args.train_out or args.val_out)

    if not args.list_categories:
        if args.out and using_split_outputs:
            parser.error("Use either --out or (--train-out and --val-out), not both")
        if not args.out and not using_split_outputs:
            parser.error("--out is required unless using --train-out/--val-out or --list-categories")
        if using_split_outputs and not (args.train_out and args.val_out):
            parser.error("--train-out and --val-out must be provided together")
        if using_split_outputs:
            if (args.val_ratio is None) == (args.val_count is None):
                parser.error("Provide exactly one of --val-ratio or --val-count when writing train/val splits")
        else:
            if args.val_ratio is not None or args.val_count is not None:
                parser.error("--val-ratio/--val-count require --train-out and --val-out")

    if args.balance_mode == "target" and args.target_per_category is None:
        parser.error("--target-per-category is required when --balance-mode target is used")

    if args.max_rows is not None and args.max_rows <= 0:
        parser.error("--max-rows must be positive")

    if args.target_per_category is not None and args.target_per_category <= 0:
        parser.error("--target-per-category must be positive")

    if args.val_ratio is not None and not (0.0 < args.val_ratio < 1.0):
        parser.error("--val-ratio must be between 0 and 1")

    if args.val_count is not None and args.val_count <= 0:
        parser.error("--val-count must be positive")

    return args


def parse_csv_set(raw: str | None) -> set[str] | None:
    if raw is None:
        return None
    values = {part.strip() for part in raw.split(",") if part.strip()}
    return values or None


def parse_category_weights(raw: str | None) -> dict[str, float]:
    if raw is None:
        return {}

    weights: dict[str, float] = {}
    for part in raw.split(","):
        part = part.strip()
        if not part:
            continue
        if "=" not in part:
            raise ValueError(f"Invalid category weight '{part}'. Expected name=value.")
        name, value = part.split("=", 1)
        name = name.strip()
        value = value.strip()
        if not name:
            raise ValueError(f"Invalid category weight '{part}'. Empty category name.")
        try:
            weight = float(value)
        except ValueError as exc:
            raise ValueError(f"Invalid weight for category '{name}': {value}") from exc
        if weight < 0:
            raise ValueError(f"Weight for category '{name}' must be >= 0")
        weights[name] = weight
    return weights


def find_jsonl_files(input_dir: Path, pattern: str, recursive: bool) -> list[Path]:
    if recursive:
        files = sorted(p for p in input_dir.rglob(pattern) if p.is_file())
    else:
        files = sorted(p for p in input_dir.glob(pattern) if p.is_file())
    return files


def iter_jsonl_examples(paths: list[Path]) -> Iterable[dict[str, Any]]:
    for path in paths:
        with path.open("r", encoding="utf-8-sig") as f:
            for line_no, line in enumerate(f, start=1):
                line = line.strip()
                if not line:
                    continue
                try:
                    obj = json.loads(line)
                except json.JSONDecodeError as exc:
                    raise ValueError(f"Invalid JSON in {path} line {line_no}: {exc}") from exc
                if not isinstance(obj, dict):
                    raise ValueError(f"Expected a JSON object in {path} line {line_no}, got {type(obj).__name__}")
                yield obj


def coerce_messages(raw: Any) -> list[dict[str, Any]]:
    if raw is None:
        return []
    if isinstance(raw, str):
        raw = raw.strip()
        if not raw:
            return []
        try:
            raw = json.loads(raw)
        except json.JSONDecodeError:
            return []
    if not isinstance(raw, list):
        return []

    normalized: list[dict[str, Any]] = []
    for item in raw:
        if not isinstance(item, dict):
            continue
        role = item.get("role")
        if role is None:
            continue
        content = item.get("content", "")
        normalized.append(
            {
                "role": str(role).strip(),
                "content": "" if content is None else str(content),
            }
        )
    return normalized


def joined_content(messages: Iterable[dict[str, Any]], role: str) -> str:
    parts = [m["content"].strip() for m in messages if m.get("role") == role and m.get("content")]
    return "\n\n".join(part for part in parts if part)


def detect_category_field(example: dict[str, Any], preferred: str) -> str | None:
    if preferred != "auto":
        return preferred if preferred in example else None
    for field in AUTO_CATEGORY_FIELDS:
        if field in example:
            return field
    return None


def get_category(example: dict[str, Any], field: str | None) -> str:
    if not field:
        return "uncategorized"
    value = example.get(field)
    if value is None:
        return "uncategorized"
    text = str(value).strip()
    return text or "uncategorized"


def example_to_record(example: dict[str, Any], *, skip_missing_assistant: bool) -> dict[str, Any] | None:
    # Support already-normalized prompt/completion arrays if present.
    prompt_messages = coerce_messages(example.get("prompt"))
    completion_messages = coerce_messages(example.get("completion"))

    if prompt_messages or completion_messages:
        cleaned_prompt: list[dict[str, str]] = []
        for msg in prompt_messages:
            role = msg.get("role", "")
            content = str(msg.get("content", "")).strip()
            if role in {"system", "user"} and content:
                cleaned_prompt.append({"role": role, "content": content})

        cleaned_completion: list[dict[str, str]] = []
        for msg in completion_messages:
            role = msg.get("role", "")
            content = str(msg.get("content", "")).strip()
            if role == "assistant" and content:
                cleaned_completion.append({"role": role, "content": content})

        has_user = any(m["role"] == "user" for m in cleaned_prompt)
        if not has_user:
            return None
        if skip_missing_assistant and not cleaned_completion:
            return None

        return {"prompt": cleaned_prompt, "completion": cleaned_completion}

    # Fallback to the original raw-message format.
    messages = coerce_messages(example.get("messages"))
    system_text = joined_content(messages, "system")
    user_text = joined_content(messages, "user")
    assistant_text = joined_content(messages, "assistant")

    if not system_text:
        system_text = str(example.get("system", "") or "").strip()
    if not user_text:
        user_text = str(example.get("prompt", "") or "").strip()
    if not assistant_text:
        assistant_text = str(example.get("completion", "") or "").strip()

    if not user_text:
        return None
    if skip_missing_assistant and not assistant_text:
        return None

    prompt_out: list[dict[str, str]] = []
    if system_text:
        prompt_out.append({"role": "system", "content": system_text})
    prompt_out.append({"role": "user", "content": user_text})

    completion_out: list[dict[str, str]] = []
    if assistant_text:
        completion_out.append({"role": "assistant", "content": assistant_text})

    return {"prompt": prompt_out, "completion": completion_out}


def compute_base_quota(counts: list[int], mode: str, target: int | None) -> int:
    if not counts:
        return 0
    if mode == "none":
        return 0
    if mode == "min":
        return min(counts)
    if mode == "max":
        return max(counts)
    if mode == "mean":
        return round(sum(counts) / len(counts))
    if mode == "median":
        return round(median(counts))
    if mode == "target":
        assert target is not None
        return target
    raise ValueError(f"Unknown balance mode: {mode}")


def quotas_for_categories(
        grouped: dict[str, list[dict[str, Any]]],
        *,
        mode: str,
        target_per_category: int | None,
        category_weights: dict[str, float],
) -> dict[str, int]:
    counts = [len(rows) for rows in grouped.values()]
    base = compute_base_quota(counts, mode, target_per_category)

    quotas: dict[str, int] = {}
    for category, rows in grouped.items():
        if mode == "none":
            quotas[category] = len(rows)
            continue
        weight = category_weights.get(category, 1.0)
        quotas[category] = max(0, math.floor(base * weight))
    return quotas


def select_rows_by_category(
        grouped: dict[str, list[dict[str, Any]]],
        *,
        quotas: dict[str, int],
        allow_oversample: bool,
        rng: random.Random,
) -> dict[str, list[dict[str, Any]]]:
    selected: dict[str, list[dict[str, Any]]] = {}

    for category, rows in grouped.items():
        quota = quotas.get(category, 0)
        if quota <= 0:
            continue

        if quota >= len(rows):
            chosen = list(rows)
            if allow_oversample and quota > len(rows):
                chosen.extend(rng.choices(rows, k=quota - len(rows)))
        else:
            chosen = rng.sample(rows, quota)

        selected[category] = chosen

    return selected


def flatten_grouped(grouped: dict[str, list[dict[str, Any]]]) -> list[tuple[str, dict[str, Any]]]:
    flat: list[tuple[str, dict[str, Any]]] = []
    for category, rows in grouped.items():
        for row in rows:
            flat.append((category, row))
    return flat


def regroup_pairs(pairs: list[tuple[str, dict[str, Any]]]) -> dict[str, list[dict[str, Any]]]:
    grouped: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for category, row in pairs:
        grouped[category].append(row)
    return grouped


def allocate_by_largest_remainder(
        capacities: dict[str, int],
        exact_targets: dict[str, float],
        total_target: int,
) -> dict[str, int]:
    total_capacity = sum(capacities.values())
    total_target = max(0, min(total_target, total_capacity))

    allocations: dict[str, int] = {}
    remainders: list[tuple[float, str]] = []

    used = 0
    for category, capacity in capacities.items():
        exact = min(float(capacity), max(0.0, exact_targets.get(category, 0.0)))
        base = min(capacity, math.floor(exact))
        allocations[category] = base
        used += base
        remainders.append((exact - base, category))

    remaining = total_target - used
    remainders.sort(reverse=True)

    idx = 0
    while remaining > 0:
        progressed = False
        for _, category in remainders:
            if remaining <= 0:
                break
            if allocations[category] < capacities[category]:
                allocations[category] += 1
                remaining -= 1
                progressed = True
        if not progressed:
            break
        idx += 1
        if idx > total_capacity + 1:
            break

    return allocations


def compute_val_counts(
        grouped: dict[str, list[dict[str, Any]]],
        *,
        val_ratio: float | None,
        val_count: int | None,
) -> dict[str, int]:
    capacities = {category: len(rows) for category, rows in grouped.items()}
    total = sum(capacities.values())

    if total == 0:
        return {category: 0 for category in grouped}

    if val_ratio is not None:
        exact_targets = {category: len(rows) * val_ratio for category, rows in grouped.items()}
        total_target = round(total * val_ratio)
        return allocate_by_largest_remainder(capacities, exact_targets, total_target)

    assert val_count is not None
    total_target = min(val_count, total)
    exact_targets = {
        category: (len(rows) / total) * total_target
        for category, rows in grouped.items()
    }
    return allocate_by_largest_remainder(capacities, exact_targets, total_target)


def stratified_split(
        grouped: dict[str, list[dict[str, Any]]],
        *,
        val_ratio: float | None,
        val_count: int | None,
        rng: random.Random,
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    val_counts = compute_val_counts(grouped, val_ratio=val_ratio, val_count=val_count)

    train: list[dict[str, Any]] = []
    val: list[dict[str, Any]] = []

    for category, rows in grouped.items():
        rows_copy = list(rows)
        rng.shuffle(rows_copy)
        n_val = val_counts.get(category, 0)
        val.extend(rows_copy[:n_val])
        train.extend(rows_copy[n_val:])

    rng.shuffle(train)
    rng.shuffle(val)
    return train, val


def write_jsonl(path: Path, rows: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as f:
        for row in rows:
            f.write(json.dumps(row, ensure_ascii=False) + "\n")


def main() -> None:
    args = parse_args()
    rng = random.Random(args.seed)

    include_categories = parse_csv_set(args.include_categories)
    exclude_categories = parse_csv_set(args.exclude_categories)
    category_weights = parse_category_weights(args.category_weights)

    input_dir = Path(args.input_dir)
    if not input_dir.exists():
        raise SystemExit(f"Input directory does not exist: {input_dir}")
    if not input_dir.is_dir():
        raise SystemExit(f"Input path is not a directory: {input_dir}")

    files = find_jsonl_files(input_dir, args.pattern, args.recursive)
    if not files:
        raise SystemExit(f"No files matched pattern '{args.pattern}' under {input_dir}")

    print(f"Found {len(files)} input file(s)")

    records_by_category: dict[str, list[dict[str, Any]]] = defaultdict(list)
    skipped = 0
    seen = 0
    category_field: str | None = None

    for example in iter_jsonl_examples(files):
        seen += 1

        if category_field is None:
            category_field = detect_category_field(example, args.category_field)

        category = get_category(example, category_field)

        if include_categories is not None and category not in include_categories:
            continue
        if exclude_categories is not None and category in exclude_categories:
            continue

        record = example_to_record(example, skip_missing_assistant=args.skip_missing_assistant)
        if record is None:
            skipped += 1
            continue

        records_by_category[category].append(record)

        if seen % 5000 == 0:
            current = sum(len(v) for v in records_by_category.values())
            print(f"Processed {seen} rows | kept={current} skipped={skipped}")

    counts = Counter({category: len(rows) for category, rows in records_by_category.items()})

    if args.list_categories:
        for category in sorted(counts):
            print(f"{category}\t{counts[category]}")
        if skipped:
            print(f"# skipped\t{skipped}")
        return

    quotas = quotas_for_categories(
        records_by_category,
        mode=args.balance_mode,
        target_per_category=args.target_per_category,
        category_weights=category_weights,
    )

    selected_by_category = select_rows_by_category(
        records_by_category,
        quotas=quotas,
        allow_oversample=args.allow_oversample,
        rng=rng,
    )

    selected_pairs = flatten_grouped(selected_by_category)

    if args.shuffle:
        rng.shuffle(selected_pairs)

    if args.max_rows is not None:
        selected_pairs = selected_pairs[:args.max_rows]

    selected_trimmed = regroup_pairs(selected_pairs)

    print(f"Detected category field: {category_field or 'none'}")
    print("Input counts by category:")
    for category in sorted(counts):
        print(f"  {category}: {counts[category]}")

    if args.balance_mode != "none":
        print("Requested quotas by category:")
        for category in sorted(quotas):
            print(f"  {category}: {quotas[category]}")

    total_selected = sum(len(v) for v in selected_trimmed.values())

    if args.train_out and args.val_out:
        train_rows, val_rows = stratified_split(
            selected_trimmed,
            val_ratio=args.val_ratio,
            val_count=args.val_count,
            rng=rng,
        )

        train_path = Path(args.train_out)
        val_path = Path(args.val_out)
        write_jsonl(train_path, train_rows)
        write_jsonl(val_path, val_rows)

        print(f"Done. Wrote {len(train_rows)} train rows to {train_path}")
        print(f"Done. Wrote {len(val_rows)} val rows to {val_path}")
    else:
        out_rows = [row for _, row in selected_pairs]
        out_path = Path(args.out)
        write_jsonl(out_path, out_rows)
        print(f"Done. Wrote {len(out_rows)} rows to {out_path}")

    print(f"Selected total rows: {total_selected}")
    if skipped:
        print(f"Skipped {skipped} rows")


if __name__ == "__main__":
    main()