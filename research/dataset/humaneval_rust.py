#!/usr/bin/env python3
"""
Export diversoailab/humaneval-rust into:

  dataset/
    problems/<uuid>.md
    tests/<uuid>.rs
    manifest.jsonl   (uuid <-> task_id mapping)

Prompts (MD) are built from:
  - prompt  -> Spec
  - declaration/declarations -> Signature (Rust code fence)

Tests (.rs) are written from:
  - test (verbatim)

UUIDs:
  - If the row already has a 'uuid' column and it's a valid UUID, use it.
  - Otherwise generate a deterministic UUIDv5 from task_id (stable across runs).

Usage:
  pip install datasets
  python humaneval_rust.py --out dataset
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import uuid
from pathlib import Path
from typing import Any, Dict, Iterable, Optional


DATASET_NAME = "diversoailab/humaneval-rust"


def _normalize_newlines(s: str) -> str:
    return s.replace("\r\n", "\n").replace("\r", "\n")


def _collapse_ws_one_line(s: str) -> str:
    # Markdown list items behave better if "Spec" is a single line.
    return re.sub(r"\s+", " ", s.strip())


def _is_valid_uuid(s: str) -> bool:
    try:
        uuid.UUID(str(s))
        return True
    except Exception:
        return False


def _stable_uuid_from_task_id(task_id: str, namespace_uuid: uuid.UUID) -> str:
    # UUIDv5 is deterministic: same task_id -> same UUID every time.
    return str(uuid.uuid5(namespace_uuid, task_id))


def _build_problem_md(spec: str, declaration: str) -> str:
    spec_line = _collapse_ws_one_line(spec)
    decl = _normalize_newlines(declaration).strip()

    # Constraints intentionally empty; Example omitted by request.
    md = (
        f"* **Spec:** {spec_line}\n"
        f"* **Constraints:**\n"
        f"* **Signature:**\n\n"
        f"```rust\n{decl}\n```\n"
    )
    return md


def _require_column(example: Dict[str, Any], keys: Iterable[str], row_hint: str) -> str:
    for k in keys:
        if k in example and example[k] is not None:
            return str(example[k])
    raise KeyError(f"Missing required column {list(keys)} in row {row_hint}")


def export(out_dir: Path, split: str, overwrite: bool, cache_dir: Optional[Path], namespace: str) -> None:
    try:
        from datasets import load_dataset  # type: ignore
    except Exception as e:
        print(
            "Failed to import 'datasets'. Install it with:\n  pip install datasets\n",
            file=sys.stderr,
        )
        raise

    namespace_uuid = uuid.UUID(namespace)

    ds_kwargs: Dict[str, Any] = {}
    if cache_dir is not None:
        ds_kwargs["cache_dir"] = str(cache_dir)

    ds = load_dataset(DATASET_NAME, split=split, **ds_kwargs)

    problems_dir = out_dir / "problems"
    tests_dir = out_dir / "tests"
    problems_dir.mkdir(parents=True, exist_ok=True)
    tests_dir.mkdir(parents=True, exist_ok=True)

    manifest_path = out_dir / "manifest.jsonl"
    if manifest_path.exists() and (not overwrite):
        raise FileExistsError(f"{manifest_path} already exists. Use --overwrite to replace it.")

    manifest_f = manifest_path.open("w", encoding="utf-8", newline="\n")

    written = 0
    try:
        for i, ex in enumerate(ds):
            task_id = str(ex.get("task_id", f"row-{i}"))

            # Prefer an existing uuid column if present+valid; otherwise generate stable UUIDv5.
            ex_uuid = ex.get("uuid", None)
            if ex_uuid is not None and _is_valid_uuid(str(ex_uuid)):
                uid = str(uuid.UUID(str(ex_uuid)))
            else:
                uid = _stable_uuid_from_task_id(task_id, namespace_uuid)

            prompt = _require_column(ex, ["prompt"], row_hint=task_id)
            declaration = _require_column(ex, ["declarations", "declaration"], row_hint=task_id)
            test = _require_column(ex, ["test"], row_hint=task_id)

            md_text = _build_problem_md(prompt, declaration)
            rs_text = _normalize_newlines(str(test)).rstrip() + "\n"

            md_path = problems_dir / f"{uid}.md"
            rs_path = tests_dir / f"{uid}.rs"

            for p in (md_path, rs_path):
                if p.exists() and (not overwrite):
                    raise FileExistsError(f"{p} already exists. Use --overwrite to replace it.")

            with md_path.open("w", encoding="utf-8", newline="\n") as f:
                f.write(md_text)

            with rs_path.open("w", encoding="utf-8", newline="\n") as f:
                f.write(rs_text)

            manifest_f.write(
                json.dumps(
                    {
                        "uuid": uid,
                        "task_id": task_id,
                        "split": split,
                        "dataset": DATASET_NAME,
                    },
                    ensure_ascii=False,
                )
                + "\n"
            )

            written += 1
    finally:
        manifest_f.close()

    print(f"Wrote {written} problems to {problems_dir} and tests to {tests_dir}")
    print(f"Manifest: {manifest_path}")


def main() -> None:
    p = argparse.ArgumentParser(description="Export diversoailab/humaneval-rust into problems/tests files.")
    p.add_argument("--out", type=Path, default=Path("dataset"), help="Output directory (default: dataset)")
    p.add_argument("--split", type=str, default="train", help="Dataset split (default: train)")
    p.add_argument("--overwrite", action="store_true", help="Overwrite existing files")
    p.add_argument("--cache-dir", type=Path, default=None, help="Hugging Face datasets cache directory")
    p.add_argument(
        "--uuid-namespace",
        type=str,
        default=str(uuid.NAMESPACE_URL),
        help="UUID namespace used for UUIDv5 generation (default: uuid.NAMESPACE_URL)",
    )
    args = p.parse_args()

    export(
        out_dir=args.out,
        split=args.split,
        overwrite=args.overwrite,
        cache_dir=args.cache_dir,
        namespace=args.uuid_namespace,
    )


if __name__ == "__main__":
    main()