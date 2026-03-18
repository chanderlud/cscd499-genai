#!/usr/bin/env python
from __future__ import annotations

import argparse
import concurrent.futures
import json
import os
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Sequence, Tuple

from helpers import configure_logging, openrouter_generate_code


BUG_CATEGORIES = {
    "borrow_checker",
    "wrong_api",
    "type_error",
    "test_logic",
    "import_error",
    "other",
}


@dataclass
class StepPair:
    attempt: int
    failed_eval_step: Dict[str, Any]
    next_generate_step: Dict[str, Any]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Mine repair and critique datasets from verified solution steps.")
    parser.add_argument("--steps-dir", type=str, required=True, help="Path to verified-solution-steps/ directory.")
    parser.add_argument("--problems-dir", type=str, required=True, help="Path to problems/ directory containing .md files.")
    parser.add_argument("--output-dir", type=str, required=True, help="Directory to write repair.jsonl and critique.jsonl.")
    parser.add_argument(
        "--critique-model",
        type=str,
        default=None,
        help="OpenRouter model override for critique generation. Defaults to OPENROUTER_CODE_MODEL.",
    )
    parser.add_argument(
        "--skip-critique",
        action="store_true",
        help="Skip critique generation and only emit repair.jsonl.",
    )
    parser.add_argument(
        "--workers",
        type=int,
        default=4,
        help="Number of concurrent workers for critique generation.",
    )
    return parser.parse_args()


def read_json(path: Path) -> Optional[Dict[str, Any]]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    if not isinstance(payload, dict):
        return None
    return payload


def read_jsonl(path: Path) -> List[Dict[str, Any]]:
    if not path.exists():
        return []
    rows: List[Dict[str, Any]] = []
    with path.open("r", encoding="utf-8") as handle:
        for raw_line in handle:
            line = raw_line.strip()
            if not line:
                continue
            try:
                payload = json.loads(line)
            except Exception:
                continue
            if isinstance(payload, dict):
                rows.append(payload)
    return rows


def write_jsonl(path: Path, rows: Iterable[Dict[str, Any]]) -> None:
    with path.open("w", encoding="utf-8") as handle:
        for row in rows:
            handle.write(json.dumps(row, ensure_ascii=False) + "\n")


def _as_bool(value: Any) -> bool:
    return bool(value)


def _as_int(value: Any) -> int:
    try:
        return int(value)
    except (TypeError, ValueError):
        return 0


def build_feedback(eval_result: Dict[str, Any], max_chars: int = 3000) -> Tuple[str, int]:
    build = eval_result.get("build") if isinstance(eval_result.get("build"), dict) else {}
    clippy = eval_result.get("clippy") if isinstance(eval_result.get("clippy"), dict) else {}
    tests = eval_result.get("tests") if isinstance(eval_result.get("tests"), dict) else {}

    sections: List[str] = []
    error_count = 0

    for stage_name, stage in (("build", build), ("clippy", clippy)):
        diagnostics = stage.get("diagnostics") if isinstance(stage.get("diagnostics"), dict) else {}
        items = diagnostics.get("items") if isinstance(diagnostics.get("items"), list) else []
        rendered_chunks: List[str] = []
        for item in items:
            if not isinstance(item, dict):
                continue
            rendered = item.get("rendered")
            message = item.get("message")
            if isinstance(rendered, str) and rendered.strip():
                rendered_chunks.append(rendered.strip())
            elif isinstance(message, str) and message.strip():
                rendered_chunks.append(message.strip())
        if rendered_chunks:
            sections.append(f"[{stage_name} diagnostics]\n" + "\n\n".join(rendered_chunks[:12]))
        error_count += _as_int(diagnostics.get("errors"))

    tests_info = tests.get("tests") if isinstance(tests.get("tests"), dict) else {}
    failed_details = tests_info.get("failed_details") if isinstance(tests_info.get("failed_details"), list) else []
    failed_sections: List[str] = []
    for detail in failed_details:
        if not isinstance(detail, dict):
            continue
        name = str(detail.get("name") or "<unknown_test>")
        panic = str(detail.get("panic_message") or "").strip()
        output = str(detail.get("output") or "").strip()
        block: List[str] = [f"[test: {name}]"]
        if panic:
            block.append(f"Panic: {panic}")
        if output:
            block.append("Output:")
            block.extend(f"  {line}" if line else "  " for line in output.splitlines()[:40])
        failed_sections.append("\n".join(block))
    if failed_sections:
        sections.append("[test failures]\n" + "\n\n".join(failed_sections))

    failed_tests_count = _as_int(tests_info.get("failed"))
    error_count += failed_tests_count

    if not sections:
        summary_parts: List[str] = []
        if isinstance(build, dict):
            summary_parts.append(f"build_ok={_as_bool(build.get('ok'))}")
        if isinstance(clippy, dict):
            summary_parts.append(f"clippy_ok={_as_bool(clippy.get('ok'))}")
        if isinstance(tests, dict):
            summary_parts.append(f"tests_ok={_as_bool(tests.get('ok'))}")
        sections.append("No detailed diagnostics were provided.\n" + ", ".join(summary_parts))

    feedback = "\n\n".join(sections).strip()
    if len(feedback) > max_chars:
        feedback = feedback[:max_chars] + "... [truncated]"
    return feedback, error_count


def load_problem_steps(steps_dir: Path) -> Dict[str, List[StepPair]]:
    per_problem: Dict[str, List[StepPair]] = {}
    for problem_dir in sorted(steps_dir.iterdir()):
        if not problem_dir.is_dir():
            continue

        step_payloads: List[Dict[str, Any]] = []
        for step_path in sorted(problem_dir.glob("step_*.json")):
            payload = read_json(step_path)
            if not payload:
                continue
            payload["_step_path"] = str(step_path)
            step_payloads.append(payload)
        if not step_payloads:
            continue

        step_payloads.sort(key=lambda item: _as_int(item.get("attempt")))

        final_path = problem_dir / "step_0_final.json"
        final_payload = read_json(final_path)
        if not final_payload:
            continue
        if final_payload.get("step_type") != "final":
            continue

        by_step_type_attempt: Dict[Tuple[int, str], Dict[str, Any]] = {}
        for step in step_payloads:
            attempt = _as_int(step.get("attempt"))
            step_type = str(step.get("step_type") or "")
            if step_type:
                by_step_type_attempt[(attempt, step_type)] = step

        pairs: List[StepPair] = []
        for step in step_payloads:
            if str(step.get("step_type") or "") != "eval":
                continue
            eval_result = step.get("eval_result")
            if not isinstance(eval_result, dict):
                continue
            if _as_bool(eval_result.get("ok")):
                continue
            attempt = _as_int(step.get("attempt"))
            next_generate = by_step_type_attempt.get((attempt + 1, "generate"))
            if not next_generate:
                continue
            pairs.append(StepPair(attempt=attempt, failed_eval_step=step, next_generate_step=next_generate))

        if pairs:
            per_problem[problem_dir.name] = pairs

    return per_problem


def extract_repair_pairs(
    problem_id: str,
    problem_text: str,
    step_pairs: Sequence[StepPair],
) -> List[Dict[str, Any]]:
    rows: List[Dict[str, Any]] = []
    for pair in step_pairs:
        failed_eval_step = pair.failed_eval_step
        next_generate_step = pair.next_generate_step

        failed_code = failed_eval_step.get("code")
        eval_result = failed_eval_step.get("eval_result")
        repaired_code = next_generate_step.get("code")
        if not isinstance(failed_code, str) or not failed_code.strip():
            continue
        if not isinstance(eval_result, dict):
            continue
        if not isinstance(repaired_code, str) or not repaired_code.strip():
            continue
        if failed_code.strip() == repaired_code.strip():
            continue

        feedback, error_count = build_feedback(eval_result)
        build = eval_result.get("build") if isinstance(eval_result.get("build"), dict) else {}
        tests = eval_result.get("tests") if isinstance(eval_result.get("tests"), dict) else {}

        rows.append(
            {
                "id": f"{problem_id}__repair__{pair.attempt}",
                "problem_id": problem_id,
                "type": "repair",
                "problem": problem_text,
                "failed_code": failed_code,
                "feedback": feedback,
                "corrected_code": repaired_code,
                "metadata": {
                    "attempt": pair.attempt,
                    "build_ok": _as_bool(build.get("ok")),
                    "test_ok": _as_bool(tests.get("ok")),
                    "error_count": error_count,
                },
            }
        )
    return rows


def _extract_json_blob(text: str) -> Optional[Dict[str, Any]]:
    raw = text.strip()
    if not raw:
        return None
    try:
        payload = json.loads(raw)
        if isinstance(payload, dict):
            return payload
    except Exception:
        pass

    fence_match = re.search(r"```(?:json)?\s*(\{.*?\})\s*```", raw, flags=re.DOTALL | re.IGNORECASE)
    if fence_match:
        candidate = fence_match.group(1)
        try:
            payload = json.loads(candidate)
            if isinstance(payload, dict):
                return payload
        except Exception:
            pass

    first = raw.find("{")
    last = raw.rfind("}")
    if first >= 0 and last > first:
        candidate = raw[first : last + 1]
        try:
            payload = json.loads(candidate)
            if isinstance(payload, dict):
                return payload
        except Exception:
            return None
    return None


def generate_critique(repair_row: Dict[str, Any], model_name: str) -> Dict[str, Any]:
    problem = str(repair_row.get("problem") or "")
    failed_code = str(repair_row.get("failed_code") or "")
    feedback = str(repair_row.get("feedback") or "")
    attempt = _as_int((repair_row.get("metadata") or {}).get("attempt"))
    problem_id = str(repair_row.get("problem_id") or "")

    system_prompt = (
        "You are a Rust debugging expert. "
        "Return ONLY a JSON object with keys: bug_category, diagnosis, repair_plan, key_insight. "
        "bug_category must be one of: borrow_checker, wrong_api, type_error, test_logic, import_error, other."
    )
    user_prompt = (
        "Analyze the failed candidate and test/build feedback.\n\n"
        "Problem:\n"
        f"{problem}\n\n"
        "Candidate code:\n"
        "```rust\n"
        f"{failed_code}\n"
        "```\n\n"
        "Feedback:\n"
        f"{feedback}\n"
    )
    response = openrouter_generate_code(
        [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt},
        ]
    )

    critique_payload: Dict[str, Any] = {
        "bug_category": None,
        "diagnosis": None,
        "repair_plan": None,
        "key_insight": None,
    }
    if isinstance(response, str) and response.strip():
        parsed = _extract_json_blob(response)
        if parsed:
            category = parsed.get("bug_category")
            if isinstance(category, str) and category in BUG_CATEGORIES:
                critique_payload["bug_category"] = category
            elif isinstance(category, str):
                critique_payload["bug_category"] = "other"

            for key in ("diagnosis", "repair_plan", "key_insight"):
                value = parsed.get(key)
                if isinstance(value, (str, int, float, bool)):
                    critique_payload[key] = str(value)
                elif value is not None:
                    critique_payload[key] = json.dumps(value, ensure_ascii=False)
        else:
            critique_payload["diagnosis"] = response.strip()
    else:
        critique_payload["diagnosis"] = "Critique generation failed or returned empty output."

    return {
        "id": f"{problem_id}__critique__{attempt}",
        "problem_id": problem_id,
        "type": "critique",
        "problem": problem,
        "candidate_code": failed_code,
        "feedback": feedback,
        "critique": critique_payload,
        "metadata": {
            "attempt": attempt,
            "model": model_name,
        },
    }


def load_problem_text(problems_dir: Path, problem_id: str) -> Optional[str]:
    md_path = problems_dir / f"{problem_id}.md"
    if not md_path.exists():
        return None
    try:
        return md_path.read_text(encoding="utf-8")
    except Exception:
        return None


def main() -> None:
    args = parse_args()
    configure_logging()

    steps_dir = Path(args.steps_dir)
    problems_dir = Path(args.problems_dir)
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    if args.critique_model:
        os.environ["OPENROUTER_CODE_MODEL"] = args.critique_model
    model_name = args.critique_model or os.getenv("OPENROUTER_CODE_MODEL", "")

    repair_path = output_dir / "repair.jsonl"
    critique_path = output_dir / "critique.jsonl"

    problem_steps = load_problem_steps(steps_dir)

    repair_rows: List[Dict[str, Any]] = []
    for problem_id, step_pairs in sorted(problem_steps.items(), key=lambda item: item[0]):
        problem_text = load_problem_text(problems_dir, problem_id)
        if not problem_text:
            continue
        repair_rows.extend(extract_repair_pairs(problem_id=problem_id, problem_text=problem_text, step_pairs=step_pairs))
    repair_rows.sort(key=lambda row: str(row.get("id") or ""))
    write_jsonl(repair_path, repair_rows)

    if args.skip_critique:
        return

    existing_critique_rows = read_jsonl(critique_path)
    existing_by_id: Dict[str, Dict[str, Any]] = {}
    for row in existing_critique_rows:
        row_id = row.get("id")
        if isinstance(row_id, str) and row_id:
            existing_by_id[row_id] = row

    critique_rows: List[Dict[str, Any]] = []
    pending_repairs: List[Dict[str, Any]] = []
    for repair_row in repair_rows:
        attempt = _as_int((repair_row.get("metadata") or {}).get("attempt"))
        critique_id = f"{repair_row.get('problem_id', '')}__critique__{attempt}"
        if critique_id in existing_by_id:
            critique_rows.append(existing_by_id[critique_id])
            continue
        pending_repairs.append(repair_row)

    if pending_repairs:
        with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, int(args.workers))) as executor:
            futures = [executor.submit(generate_critique, row, model_name) for row in pending_repairs]
            for future in concurrent.futures.as_completed(futures):
                critique_rows.append(future.result())

    critique_rows.sort(key=lambda row: str(row.get("id") or ""))
    write_jsonl(critique_path, critique_rows)


if __name__ == "__main__":
    main()
