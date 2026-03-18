import argparse
import concurrent.futures
import os
import random
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Tuple

import httpx

from generate_problems_agent import (
    REPAIR_PROMPT_TEMPLATE,
    SOLUTION_SYSTEM_PROMPT,
    _error_score,
    _extract_problem_md,
    _extract_title,
    _first_diagnostic_hint,
    _realign_problem,
)
from helpers import (
    LOGGER,
    StepRecorder,
    batch_rustdoc_lookup,
    build_repair_context,
    configure_logging,
    env,
    eval_server_evaluate,
    eval_server_format,
    eval_server_warmup,
    extract_rust_code_block,
    extract_symbols_from_diagnostics,
    extract_windows_api_symbols,
    openrouter_generate_code,
    preview_text,
)


FIXED_DEPENDENCIES = (
    Path(__file__).resolve().parent.parent / "rust_dependencies.md"
).read_text(encoding="utf-8")

STRATEGY_CONSTRAINT_SHIFT = "constraint_shift"
STRATEGY_API_FORM_SHIFT = "api_form_shift"
STRATEGY_ALGORITHMIC_SHIFT = "algorithmic_shift"
STRATEGY_ADVERSARIAL = "adversarial"
STRATEGY_MERGE = "merge"
STRATEGY_RANDOM = "random"

STRATEGY_DESCRIPTIONS: Dict[str, str] = {
    STRATEGY_CONSTRAINT_SHIFT: "Same core task, but with tighter constraints or complexity/resource limits.",
    STRATEGY_API_FORM_SHIFT: "Change API form (iterators/indexing, ownership/borrowing, trait bounds, Result paths).",
    STRATEGY_ALGORITHMIC_SHIFT: "Keep semantics but target a different algorithmic approach or edge-case regime.",
    STRATEGY_ADVERSARIAL: "Include a plausible-but-wrong hint/reference approach the solver must avoid.",
    STRATEGY_MERGE: "Combine 2-3 source problems into one harder composite challenge.",
}

_STRATEGY_INSTRUCTIONS: Dict[str, str] = {
    STRATEGY_CONSTRAINT_SHIFT: (
        "Tighten one or more constraints (e.g., forbid heap allocation, forbid recursion, "
        "require O(n) instead of O(n log n), require stable ordering, or require streaming/iterator input)."
    ),
    STRATEGY_API_FORM_SHIFT: (
        "Change the API surface: switch from index-based to iterator-based, add Result-returning error paths, "
        "add a trait bound, or impose an ownership/borrowing constraint."
    ),
    STRATEGY_ALGORITHMIC_SHIFT: (
        "Keep the same observable semantics but require a different algorithm or target a different edge-case regime."
    ),
    STRATEGY_ADVERSARIAL: (
        "Embed a plausible-but-wrong reference approach or a misleading partial implementation in the problem statement "
        "as a hint that the solver must recognize and avoid."
    ),
    STRATEGY_MERGE: (
        "Combine the provided source problems into one harder composite problem that requires solving both sub-tasks "
        "in a single function or module."
    ),
}

EVOLUTION_SYSTEM_PROMPT = """You are a technical problem-statement author for Rust/Win32 coding challenges.

Given one or more existing verified problem+solution pairs, produce exactly ONE NEW evolved challenge.

Required output structure:
- A `TITLE:` line with a short imperative title.
- A `PROBLEM:` sentinel on its own line.
- A Markdown block with:
  - `**Spec:**` bullet describing what the function must do
  - `**Constraints:**` bullet listing key requirements
  - `**Signature:**` bullet with a fenced Rust signature block
  - `**Example:**` bullet with a minimal usage snippet

Signature & Example rules:
- Use `windows::core::Result<T>` (or `windows::core::Result<()>`) for all fallible return types in the signature.
- Do NOT include any `use windows::...` import lines in the Signature or Example blocks.
- The Example block should show call-site usage only, not imports.

Win32 rules:
- Keep the challenge in Rust with the `windows` crate.
- Prefer W-suffix Win32 APIs where applicable.
- Do not require unsound `unsafe impl Send/Sync`.
- Keep requirements precise, verifiable, and self-contained.

Do not include any extra prose outside this structure.
"""

EVOLUTION_USER_TEMPLATE = """Evolve one NEW Rust/Win32 challenge from the source material below.

Strategy:
- key: {strategy_key}
- description: {strategy_description}

Strategy-specific requirement:
{strategy_instruction}

Already generated evolved ideas:
{previously_generated}

If there are no more meaningfully different ideas left for the selected source set, respond with exactly:
NO_MORE_IDEAS

## Source material
{source_problems}
"""

MISLEADING_HINT_SYSTEM_PROMPT = """You are generating a deliberately flawed Rust hint snippet.

Rules:
- Keep the snippet syntactically valid Rust.
- Introduce exactly one subtle, plausible bug (off-by-one, boundary condition, wrong API variant, wrong error path, etc.).
- Keep code style realistic and close to the original.
- Output only one ```rust code fence with the flawed snippet.
"""

MISLEADING_HINT_USER_TEMPLATE = """Create a subtly wrong variant of this solution:
```rust
{main_rs}
```
"""


@dataclass
class SourceProblem:
    source_id: str
    title: str
    problem_md: str
    solution_rs: str


@dataclass
class EvolvedProblemResult:
    idea: str
    problem_md: str
    main_rs: str
    last_eval: Dict[str, Any]
    verified: bool = True
    strategy: str = STRATEGY_CONSTRAINT_SHIFT
    source_ids: List[str] = None

    def __post_init__(self) -> None:
        if self.source_ids is None:
            self.source_ids = []


def _extract_source_title(problem_md: str, source_id: str) -> str:
    for line in problem_md.splitlines():
        stripped = line.strip()
        if stripped.startswith("TITLE:"):
            title = stripped[len("TITLE:") :].strip()
            if title:
                return title
            break
    for line in problem_md.splitlines():
        stripped = line.strip()
        if not stripped:
            continue
        if stripped.startswith("#"):
            stripped = stripped.lstrip("#").strip()
        if stripped:
            return stripped
    return f"Source {source_id}"


def _load_source_problems(input_dir: Path) -> List[SourceProblem]:
    problems_dir = input_dir / "problems"
    solutions_dir = input_dir / "solutions"
    if not problems_dir.is_dir() or not solutions_dir.is_dir():
        raise ValueError(
            f"Expected input dir with problems/ and solutions/ subdirectories, got: {input_dir}"
        )

    loaded: List[SourceProblem] = []
    for problem_path in sorted(problems_dir.glob("*.md")):
        source_id = problem_path.stem
        solution_path = solutions_dir / f"{source_id}.rs"
        if not solution_path.exists():
            LOGGER.warning("source_missing_solution source_id=%s path=%s", source_id, solution_path)
            continue
        try:
            problem_md = problem_path.read_text(encoding="utf-8")
            solution_rs = solution_path.read_text(encoding="utf-8")
        except OSError as exc:
            LOGGER.warning("source_read_failed source_id=%s error=%s", source_id, exc)
            continue
        title = _extract_source_title(problem_md, source_id)
        loaded.append(
            SourceProblem(
                source_id=source_id,
                title=title,
                problem_md=problem_md,
                solution_rs=solution_rs,
            )
        )

    if not loaded:
        raise ValueError(f"No source problem+solution pairs found in {input_dir}")
    return loaded


def _format_previous_ideas(previously_generated: Sequence[EvolvedProblemResult]) -> str:
    if not previously_generated:
        return "(none yet)"
    lines: List[str] = []
    for idx, item in enumerate(previously_generated, start=1):
        lines.append(f"{idx}. {item.idea} [{item.strategy}]")
    return "\n".join(lines)


def _resolve_strategy(strategy: str) -> str:
    if strategy == STRATEGY_RANDOM:
        return random.choice(list(STRATEGY_DESCRIPTIONS.keys()))
    if strategy not in STRATEGY_DESCRIPTIONS:
        raise ValueError(f"Unsupported strategy: {strategy}")
    return strategy


def _select_sources(
    all_sources: Sequence[SourceProblem],
    strategy: str,
    merge_count: int,
    source_id: Optional[str] = None,
) -> List[SourceProblem]:
    if not all_sources:
        raise ValueError("No source problems loaded.")

    by_id = {item.source_id: item for item in all_sources}
    pinned: Optional[SourceProblem] = None
    if source_id:
        pinned = by_id.get(source_id)
        if pinned is None:
            raise ValueError(f"source_id not found: {source_id}")

    if strategy == STRATEGY_MERGE:
        desired = max(2, int(merge_count))
        desired = min(desired, 3)
        desired = min(desired, len(all_sources))
        if pinned is not None:
            remaining = [item for item in all_sources if item.source_id != pinned.source_id]
            take = max(0, desired - 1)
            picked = [pinned]
            if take > 0 and remaining:
                picked.extend(random.sample(remaining, min(take, len(remaining))))
            return picked
        if len(all_sources) == 1:
            return [all_sources[0]]
        return random.sample(list(all_sources), desired)

    if pinned is not None:
        return [pinned]
    return [random.choice(list(all_sources))]


def _generate_misleading_hint(main_rs: str) -> str:
    messages = [
        {"role": "system", "content": MISLEADING_HINT_SYSTEM_PROMPT},
        {"role": "user", "content": MISLEADING_HINT_USER_TEMPLATE.format(main_rs=main_rs)},
    ]
    response = openrouter_generate_code(messages)
    if not response:
        return main_rs
    code = extract_rust_code_block(response)
    return code if code else main_rs


def _format_source_problems(selected: Sequence[SourceProblem], strategy: str) -> str:
    blocks: List[str] = []
    for idx, source in enumerate(selected, start=1):
        blocks.append(f"## Source Problem {idx} (id: {source.source_id})")
        blocks.append("### Problem Statement")
        blocks.append(source.problem_md.strip())
        blocks.append("### Reference Solution")
        blocks.append("```rust")
        blocks.append(source.solution_rs.rstrip())
        blocks.append("```")
        if strategy == STRATEGY_ADVERSARIAL:
            misleading = _generate_misleading_hint(source.solution_rs)
            blocks.append("### Misleading Hint (intentionally flawed)")
            blocks.append("```rust")
            blocks.append(misleading.rstrip())
            blocks.append("```")
        blocks.append("")
    return "\n".join(blocks).strip()


def evolve_one_problem(
    source_problems: Sequence[SourceProblem],
    strategy: str,
    merge_count: int,
    source_id: Optional[str],
    previously_generated: Sequence[EvolvedProblemResult],
    max_repair_attempts: int,
    eval_base: str,
    rustdocs_base: str,
    client: httpx.Client,
    recorder: StepRecorder,
    run_id: str,
) -> Optional[EvolvedProblemResult]:
    chosen_strategy = _resolve_strategy(strategy)
    selected_sources = _select_sources(
        all_sources=source_problems,
        strategy=chosen_strategy,
        merge_count=merge_count,
        source_id=source_id,
    )
    source_ids = [item.source_id for item in selected_sources]
    source_block = _format_source_problems(selected_sources, chosen_strategy)
    previous_list = _format_previous_ideas(previously_generated)

    ideation_prompt = EVOLUTION_USER_TEMPLATE.format(
        strategy_key=chosen_strategy,
        strategy_description=STRATEGY_DESCRIPTIONS[chosen_strategy],
        strategy_instruction=_STRATEGY_INSTRUCTIONS[chosen_strategy],
        source_problems=source_block,
        previously_generated=previous_list,
    )
    messages = [
        {"role": "system", "content": EVOLUTION_SYSTEM_PROMPT},
        {"role": "user", "content": ideation_prompt},
    ]

    ideation_response: Optional[str] = None
    for retry in range(2):
        ideation_response = openrouter_generate_code(messages)
        if ideation_response is not None:
            break
        if retry == 0:
            retry_prompt = (
                ideation_prompt
                + "\n\nPlease output `TITLE:`, then `PROBLEM:`, then the Markdown block exactly in the required structure."
            )
            messages = [
                {"role": "system", "content": EVOLUTION_SYSTEM_PROMPT},
                {"role": "user", "content": retry_prompt},
            ]

    if ideation_response is None:
        LOGGER.warning("evolve_one_problem ideation_failed run_id=%s strategy=%s", run_id, chosen_strategy)
        return None

    stripped_response = ideation_response.strip()
    if "NO_MORE_IDEAS" in stripped_response:
        recorder.record_step(
            attempt=1,
            step_type="ideate_opt_out",
            code="",
            eval_result=None,
            extra_context={
                "strategy": chosen_strategy,
                "source_ids": source_ids,
                "response": preview_text(stripped_response, limit=300),
            },
        )
        return None

    idea = _extract_title(ideation_response)
    problem_md = _extract_problem_md(ideation_response)
    if not problem_md:
        recorder.record_step(
            attempt=1,
            step_type="ideate_generate",
            code="",
            eval_result=None,
            extra_context={
                "idea": idea,
                "strategy": chosen_strategy,
                "source_ids": source_ids,
                "response_preview": preview_text(ideation_response, limit=500),
            },
        )
        LOGGER.warning(
            "evolve_one_problem empty_problem_md run_id=%s strategy=%s idea=%r",
            run_id,
            chosen_strategy,
            idea,
        )
        return None

    recorder.record_step(
        attempt=1,
        step_type="ideate_generate",
        code="",
        eval_result=None,
        extra_context={
            "idea": idea,
            "strategy": chosen_strategy,
            "source_ids": source_ids,
            "problem_preview": preview_text(problem_md, limit=500),
        },
    )

    best_code = ""
    best_eval: Dict[str, Any] = {}
    best_score = 10**9
    previous_code = ""
    same_streak = 0
    repair_context = ""

    for attempt in range(1, max_repair_attempts + 1):
        user_prompt = problem_md if attempt == 1 else REPAIR_PROMPT_TEMPLATE.format(context=repair_context)
        solve_messages = [
            {"role": "system", "content": SOLUTION_SYSTEM_PROMPT},
            {"role": "user", "content": user_prompt},
        ]

        response_text: Optional[str] = None
        for retry in range(2):
            response_text = openrouter_generate_code(solve_messages)
            if response_text is not None:
                break
            if retry == 0:
                retry_prompt = user_prompt + "\n\nPlease generate code. Output only a ```rust code block."
                solve_messages = [
                    {"role": "system", "content": SOLUTION_SYSTEM_PROMPT},
                    {"role": "user", "content": retry_prompt},
                ]

        if response_text is None:
            LOGGER.warning(
                "evolve_one_problem solution_generation_failed run_id=%s attempt=%s idea=%r strategy=%s",
                run_id,
                attempt,
                idea,
                chosen_strategy,
            )
            continue

        code = extract_rust_code_block(response_text)
        if code is None:
            recorder.record_step(
                attempt=attempt,
                step_type="no_code",
                code="",
                eval_result=None,
                extra_context={
                    "idea": idea,
                    "strategy": chosen_strategy,
                    "source_ids": source_ids,
                    "response_preview": preview_text(response_text, limit=500),
                },
            )
            repair_context = (
                "## Build/Test Results\n"
                "No Rust code block was generated in the previous attempt.\n\n"
                "## Repair Instructions\n"
                "- Output the complete src/main.rs in a single ```rust code fence.\n"
            )
            continue

        recorder.record_step(
            attempt=attempt,
            step_type="generate",
            code=code,
            eval_result=None,
            extra_context={
                "idea": idea,
                "strategy": chosen_strategy,
                "source_ids": source_ids,
                "phase": "initial" if attempt == 1 else "repair",
            },
        )

        symbols = extract_windows_api_symbols(code)
        rustdoc_info = ""
        try:
            rustdoc_info = batch_rustdoc_lookup(symbols, rustdocs_base, client)
        except Exception as exc:
            LOGGER.warning("evolve_one_problem rustdoc_lookup_failed run_id=%s attempt=%s error=%s", run_id, attempt, exc)

        eval_result: Dict[str, Any] = {}
        eval_error: Optional[Exception] = None
        for eval_try in range(2):
            try:
                eval_result = eval_server_evaluate(
                    main_rs=code,
                    unit_tests_private="",
                    fixed_deps=FIXED_DEPENDENCIES,
                    eval_base=eval_base,
                    client=client,
                    run_tests=False,
                )
                eval_error = None
                break
            except (httpx.TimeoutException, httpx.HTTPStatusError) as exc:
                eval_error = exc
                LOGGER.warning(
                    "evolve_one_problem eval_retry run_id=%s attempt=%s eval_try=%s error=%s",
                    run_id,
                    attempt,
                    eval_try + 1,
                    exc,
                )
            except Exception as exc:
                eval_error = exc
                LOGGER.warning(
                    "evolve_one_problem eval_failed run_id=%s attempt=%s error=%s",
                    run_id,
                    attempt,
                    exc,
                )
                break

        if eval_error is not None and not eval_result:
            recorder.record_step(
                attempt=attempt,
                step_type="eval_error",
                code=code,
                eval_result=None,
                extra_context={
                    "idea": idea,
                    "strategy": chosen_strategy,
                    "source_ids": source_ids,
                    "error": str(eval_error),
                },
            )
            repair_context = (
                "## Build/Test Results\n"
                f"Evaluator request failed: {eval_error}\n\n"
                "## Repair Instructions\n"
                "- Keep the same approach and output valid Rust in a single fence.\n"
            )
            continue

        recorder.record_step(
            attempt=attempt,
            step_type="eval",
            code=code,
            eval_result=eval_result,
            extra_context={
                "idea": idea,
                "strategy": chosen_strategy,
                "source_ids": source_ids,
                "symbols": symbols,
                "rustdoc_info": rustdoc_info,
            },
        )

        score = _error_score(eval_result)
        if score < best_score:
            best_score = score
            best_code = code
            best_eval = eval_result

        if eval_result.get("ok") is True:
            formatted = None
            try:
                formatted = eval_server_format(code, eval_base, client)
            except Exception as exc:
                LOGGER.warning("evolve_one_problem format_failed run_id=%s attempt=%s error=%s", run_id, attempt, exc)
                formatted = None

            if formatted and formatted.strip() != code.strip():
                recorder.record_step(
                    attempt=attempt,
                    step_type="format",
                    code=formatted,
                    eval_result=None,
                    extra_context={"idea": idea, "strategy": chosen_strategy, "source_ids": source_ids},
                )
                try:
                    formatted_eval = eval_server_evaluate(
                        main_rs=formatted,
                        unit_tests_private="",
                        fixed_deps=FIXED_DEPENDENCIES,
                        eval_base=eval_base,
                        client=client,
                        run_tests=False,
                    )
                except Exception as exc:
                    LOGGER.warning(
                        "evolve_one_problem formatted_recheck_failed run_id=%s attempt=%s error=%s",
                        run_id,
                        attempt,
                        exc,
                    )
                    formatted_eval = None

                if isinstance(formatted_eval, dict) and formatted_eval.get("ok") is True:
                    aligned_problem_md = _realign_problem(problem_md, formatted.rstrip() + "\n", run_id)
                    recorder.record_step(
                        attempt=attempt,
                        step_type="realign",
                        code=formatted.rstrip() + "\n",
                        eval_result=None,
                        extra_context={
                            "idea": idea,
                            "strategy": chosen_strategy,
                            "source_ids": source_ids,
                            "original_problem_preview": preview_text(problem_md, 300),
                            "aligned_problem_preview": preview_text(aligned_problem_md, 300),
                        },
                    )
                    return EvolvedProblemResult(
                        idea=idea,
                        problem_md=aligned_problem_md,
                        main_rs=formatted.rstrip() + "\n",
                        last_eval=formatted_eval,
                        strategy=chosen_strategy,
                        source_ids=source_ids,
                    )

            aligned_problem_md = _realign_problem(problem_md, code.rstrip() + "\n", run_id)
            recorder.record_step(
                attempt=attempt,
                step_type="realign",
                code=code.rstrip() + "\n",
                eval_result=None,
                extra_context={
                    "idea": idea,
                    "strategy": chosen_strategy,
                    "source_ids": source_ids,
                    "original_problem_preview": preview_text(problem_md, 300),
                    "aligned_problem_preview": preview_text(aligned_problem_md, 300),
                },
            )
            return EvolvedProblemResult(
                idea=idea,
                problem_md=aligned_problem_md,
                main_rs=code.rstrip() + "\n",
                last_eval=eval_result,
                strategy=chosen_strategy,
                source_ids=source_ids,
            )

        diagnostic_symbols = extract_symbols_from_diagnostics(eval_result)
        targeted_info = ""
        if diagnostic_symbols:
            try:
                targeted_info = batch_rustdoc_lookup(diagnostic_symbols, rustdocs_base, client)
            except Exception as exc:
                LOGGER.warning(
                    "evolve_one_problem targeted_lookup_failed run_id=%s attempt=%s error=%s",
                    run_id,
                    attempt,
                    exc,
                )

        combined_info_parts = [part for part in [rustdoc_info, targeted_info] if part.strip()]
        combined_info = "\n\n".join(combined_info_parts)
        repair_context = build_repair_context(
            eval_result=eval_result,
            main_rs=code,
            rustdoc_info=combined_info,
            problem_text=problem_md,
        )

        if code.strip() == previous_code.strip():
            same_streak += 1
        else:
            same_streak = 0
        previous_code = code

        if same_streak >= 2:
            hint = _first_diagnostic_hint(eval_result)
            repair_context += (
                "\n\nWARNING: Your previous repair attempt returned identical code. "
                "You MUST make a different change this time. Focus on: "
                f"{hint}"
            )

    if best_code.strip():
        LOGGER.warning(
            "evolve_one_problem exhausted_attempts returning_best run_id=%s strategy=%s idea=%r best_score=%s",
            run_id,
            chosen_strategy,
            idea,
            best_score,
        )
        recorder.record_step(
            attempt=max_repair_attempts + 1,
            step_type="best_effort",
            code=best_code,
            eval_result=best_eval,
            extra_context={
                "idea": idea,
                "strategy": chosen_strategy,
                "source_ids": source_ids,
                "best_score": best_score,
            },
        )
        return EvolvedProblemResult(
            idea=idea,
            problem_md=problem_md,
            main_rs=best_code.rstrip() + "\n",
            last_eval=best_eval,
            verified=False,
            strategy=chosen_strategy,
            source_ids=source_ids,
        )

    raise RuntimeError(
        f"Failed to solve evolved idea {idea!r} within {max_repair_attempts} attempts and no code was produced."
    )


def _write_result(output_dir: Path, result: EvolvedProblemResult) -> None:
    item_id = str(uuid.uuid4())
    problem_path = output_dir / "problems" / f"{item_id}.md"
    solution_path = output_dir / "solutions" / f"{item_id}.rs"
    problem_path.write_text(result.problem_md, encoding="utf-8")
    solution_path.write_text(result.main_rs, encoding="utf-8")
    LOGGER.info(
        "process_evolution_batch saved id=%s idea=%r strategy=%s sources=%s",
        item_id,
        result.idea,
        result.strategy,
        ",".join(result.source_ids),
    )


def _evolve_single_with_fresh_client(
    source_problems: Sequence[SourceProblem],
    strategy: str,
    merge_count: int,
    source_id: Optional[str],
    previously_generated: Sequence[EvolvedProblemResult],
    max_repair_attempts: int,
    eval_base: str,
    rustdocs_base: str,
    output_dir: Optional[Path],
) -> Optional[EvolvedProblemResult]:
    run_id = uuid.uuid4().hex[:8]
    recorder = StepRecorder(run_id=run_id, output_dir=output_dir)
    with httpx.Client(timeout=120.0) as client:
        eval_server_warmup(eval_base, client)
        return evolve_one_problem(
            source_problems=source_problems,
            strategy=strategy,
            merge_count=merge_count,
            source_id=source_id,
            previously_generated=previously_generated,
            max_repair_attempts=max_repair_attempts,
            eval_base=eval_base,
            rustdocs_base=rustdocs_base,
            client=client,
            recorder=recorder,
            run_id=run_id,
        )


def process_evolution_batch(
    source_problems: Sequence[SourceProblem],
    strategy: str,
    output_dir: Optional[Path],
    max_problems: int,
    max_repair_attempts: int,
    merge_count: int,
    source_id: Optional[str],
    overwrite: bool,
    workers: int,
) -> List[EvolvedProblemResult]:
    if output_dir is not None:
        output_dir.mkdir(parents=True, exist_ok=True)
        (output_dir / "problems").mkdir(parents=True, exist_ok=True)
        (output_dir / "solutions").mkdir(parents=True, exist_ok=True)
        has_existing = any((output_dir / "problems").glob("*.md")) or any((output_dir / "solutions").glob("*.rs"))
        if has_existing and not overwrite:
            LOGGER.info("Skipping generation (output exists, use --overwrite to regenerate).")
            return []

    eval_base = env("RUST_EVAL_BASE_URL", "http://127.0.0.1:3002")
    rustdocs_base = env("RUSTDOCS_BASE_URL", "http://127.0.0.1:3001")
    results: List[EvolvedProblemResult] = []

    if max(1, int(workers)) == 1:
        run_id = uuid.uuid4().hex[:8]
        recorder = StepRecorder(run_id=run_id, output_dir=output_dir)
        with httpx.Client(timeout=120.0) as client:
            eval_server_warmup(eval_base, client)
            for _ in range(max(0, max_problems)):
                try:
                    generated = evolve_one_problem(
                        source_problems=source_problems,
                        strategy=strategy,
                        merge_count=merge_count,
                        source_id=source_id,
                        previously_generated=results,
                        max_repair_attempts=max_repair_attempts,
                        eval_base=eval_base,
                        rustdocs_base=rustdocs_base,
                        client=client,
                        recorder=recorder,
                        run_id=run_id,
                    )
                except Exception as exc:
                    LOGGER.warning("process_evolution_batch evolve_one_problem_failed run_id=%s error=%s", run_id, exc)
                    continue

                if generated is None:
                    LOGGER.info("process_evolution_batch ideation_opt_out run_id=%s generated=%s", run_id, len(results))
                    break
                if not generated.verified:
                    LOGGER.warning(
                        "process_evolution_batch best_effort_skipped run_id=%s idea=%r strategy=%s eval=%s",
                        run_id,
                        generated.idea,
                        generated.strategy,
                        preview_text(generated.last_eval, limit=300),
                    )
                    continue
                results.append(generated)
                if output_dir is not None:
                    _write_result(output_dir, generated)
        return results

    workers_count = max(1, int(workers))
    attempts_budget = max(max_problems, workers_count) * 3
    in_flight: Dict[concurrent.futures.Future[Optional[EvolvedProblemResult]], int] = {}
    submitted = 0

    with concurrent.futures.ThreadPoolExecutor(max_workers=workers_count) as executor:
        while len(results) < max_problems:
            while (
                len(in_flight) < workers_count
                and submitted < attempts_budget
                and len(results) + len(in_flight) < max_problems
            ):
                future = executor.submit(
                    _evolve_single_with_fresh_client,
                    source_problems,
                    strategy,
                    merge_count,
                    source_id,
                    list(results),
                    max_repair_attempts,
                    eval_base,
                    rustdocs_base,
                    output_dir,
                )
                in_flight[future] = submitted
                submitted += 1

            if not in_flight:
                break

            done, _ = concurrent.futures.wait(
                in_flight.keys(),
                return_when=concurrent.futures.FIRST_COMPLETED,
            )
            for future in done:
                in_flight.pop(future, None)
                try:
                    generated = future.result()
                except Exception as exc:
                    LOGGER.warning("process_evolution_batch worker_failed error=%s", exc)
                    continue
                if generated is None:
                    continue
                if not generated.verified:
                    LOGGER.warning(
                        "process_evolution_batch best_effort_skipped idea=%r strategy=%s eval=%s",
                        generated.idea,
                        generated.strategy,
                        preview_text(generated.last_eval, limit=300),
                    )
                    continue
                results.append(generated)
                if output_dir is not None:
                    _write_result(output_dir, generated)
                if len(results) >= max_problems:
                    break

            if submitted >= attempts_budget and not in_flight:
                break

    return results


if __name__ == "__main__":
    configure_logging()

    parser = argparse.ArgumentParser(
        description="Evolve verified Win32 problems into harder variants with compile-validated Rust solutions."
    )
    parser.add_argument(
        "--input-dir",
        required=True,
        help="Path to input directory containing problems/ and solutions/ subdirectories.",
    )
    parser.add_argument(
        "--output-dir",
        default="./evolved_out",
        help="Root output directory for evolved problems/solutions.",
    )
    parser.add_argument(
        "--strategy",
        default=STRATEGY_CONSTRAINT_SHIFT,
        choices=[
            STRATEGY_CONSTRAINT_SHIFT,
            STRATEGY_API_FORM_SHIFT,
            STRATEGY_ALGORITHMIC_SHIFT,
            STRATEGY_ADVERSARIAL,
            STRATEGY_MERGE,
            STRATEGY_RANDOM,
        ],
        help="Evolution strategy key. Use random to choose one per iteration.",
    )
    parser.add_argument(
        "--max-problems",
        type=int,
        default=10,
        help="Number of evolved problems to generate.",
    )
    parser.add_argument(
        "--max-attempts",
        type=int,
        default=8,
        help="Max repair attempts per generated solution.",
    )
    parser.add_argument(
        "--merge-count",
        type=int,
        default=2,
        help="Number of source problems to merge (only used with merge strategy).",
    )
    parser.add_argument(
        "--source-id",
        default=None,
        help="Optional source problem UUID to pin.",
    )
    parser.add_argument(
        "--overwrite",
        action="store_true",
        help="Re-generate even if output exists.",
    )
    parser.add_argument(
        "--workers",
        type=int,
        default=1,
        help="Maximum number of concurrent workers.",
    )
    args = parser.parse_args()

    input_dir = Path(args.input_dir)
    output_dir = Path(args.output_dir)
    source_problems = _load_source_problems(input_dir)

    LOGGER.info(
        "evolve_problems_agent start input_dir=%s sources=%s output_dir=%s strategy=%s max_problems=%s max_attempts=%s merge_count=%s source_id=%s overwrite=%s workers=%s cwd=%s",
        input_dir,
        len(source_problems),
        output_dir,
        args.strategy,
        args.max_problems,
        args.max_attempts,
        args.merge_count,
        args.source_id,
        args.overwrite,
        args.workers,
        os.getcwd(),
    )

    results = process_evolution_batch(
        source_problems=source_problems,
        strategy=args.strategy,
        output_dir=output_dir,
        max_problems=args.max_problems,
        max_repair_attempts=args.max_attempts,
        merge_count=args.merge_count,
        source_id=args.source_id,
        overwrite=args.overwrite,
        workers=max(1, int(args.workers)),
    )
    LOGGER.info("evolve_problems_agent completed generated=%s", len(results))
