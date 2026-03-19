
#!/usr/bin/env python
"""
QLoRA SFT training script for Qwen2.5-Coder on Rust problem/solution data.

Supported input formats (JSONL or JSON):
1) Conversational prompt-completion:
   {
     "prompt": [{"role": "system", "content": "..."},
                {"role": "user", "content": "..."}],
     "completion": [{"role": "assistant", "content": "..."}]
   }

2) Standard prompt-completion:
   {
     "prompt": "Write a Rust function that ...",
     "completion": "fn main() { ... }"
   }

3) Simple field pair (configure with --user-field and --assistant-field):
   {
     "problem": "Write a Rust function that ...",
     "solution": "fn main() { ... }"
   }

Example:
    accelerate launch train.py ^
      --train-file data/train.jsonl ^
      --eval-file data/valid.jsonl ^
      --output-dir outputs/qwen25-coder-rust-7b-qlora ^
      --model-name Qwen/Qwen2.5-Coder-7B-Instruct ^
      --max-length 2048 ^
      --per-device-train-batch-size 1 ^
      --gradient-accumulation-steps 16 ^
      --learning-rate 1e-4 ^
      --num-train-epochs 3 ^
      --use-4bit
"""

from __future__ import annotations

import argparse
import json
import math
import os
import random
from pathlib import Path
from typing import Any, Dict, List, Optional

import torch
from datasets import Dataset, DatasetDict, concatenate_datasets, load_dataset
from transformers import (
    AutoModelForCausalLM,
    AutoTokenizer,
    BitsAndBytesConfig,
    set_seed,
)

from peft import LoraConfig, prepare_model_for_kbit_training
from trl import SFTConfig, SFTTrainer

FIM_PREFIX_TOKEN = "<|fim_prefix|>"
FIM_SUFFIX_TOKEN = "<|fim_suffix|>"
FIM_MIDDLE_TOKEN = "<|fim_middle|>"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Train Qwen2.5-Coder with QLoRA SFT on Rust problem/solution data.")

    # Data
    parser.add_argument("--train-file", type=str, required=True, help="Path to training JSON/JSONL file.")
    parser.add_argument("--eval-file", type=str, default=None, help="Optional path to validation JSON/JSONL file.")
    parser.add_argument(
        "--validation-split-ratio",
        type=float,
        default=0.0,
        help="If --eval-file is omitted, carve validation from train with this ratio (e.g. 0.05).",
    )
    parser.add_argument("--user-field", type=str, default="problem", help="Field name for user/problem text.")
    parser.add_argument("--assistant-field", type=str, default="solution", help="Field name for assistant/solution text.")
    parser.add_argument("--system-prompt", type=str, default="", help="Optional system prompt to prepend.")
    parser.add_argument(
        "--dataset-num-proc",
        type=int,
        default=None,
        help="Processes for dataset preprocessing. Leave unset on Windows if you hit multiprocessing issues.",
    )
    parser.add_argument(
        "--shuffle-train",
        action="store_true",
        help="Shuffle training rows before training. The trainer also shuffles by default when sampling batches.",
    )
    parser.add_argument(
        "--fim-augmentation-ratio",
        type=float,
        default=0.05,
        help=(
            "Fraction of training examples to duplicate as FIM variants. "
            "Recommended range: 0.05-0.10. Set to 0.0 to disable FIM augmentation."
        ),
    )

    # Model
    parser.add_argument("--model-name", type=str, default="Qwen/Qwen2.5-Coder-7B-Instruct")
    parser.add_argument("--attn-implementation", type=str, default="sdpa", choices=["sdpa", "eager", "flash_attention_2"])
    parser.add_argument("--trust-remote-code", action="store_true")
    parser.add_argument("--use-4bit", action="store_true", help="Enable QLoRA 4-bit loading with bitsandbytes.")
    parser.add_argument("--no-use-4bit", dest="use_4bit", action="store_false")
    parser.set_defaults(use_4bit=True)
    parser.add_argument("--disable-bf16", action="store_true", help="Force fp16 instead of bf16.")
    parser.add_argument("--gradient-checkpointing", action="store_true")
    parser.add_argument("--no-gradient-checkpointing", dest="gradient_checkpointing", action="store_false")
    parser.set_defaults(gradient_checkpointing=True)

    # LoRA
    parser.add_argument("--lora-r", type=int, default=16)
    parser.add_argument("--lora-alpha", type=int, default=32)
    parser.add_argument("--lora-dropout", type=float, default=0.05)
    parser.add_argument(
        "--target-modules",
        type=str,
        default="all-linear",
        help='Either "all-linear" or a comma-separated list like q_proj,k_proj,v_proj,o_proj',
    )

    # Training
    parser.add_argument("--output-dir", type=str, required=True)
    parser.add_argument("--max-length", type=int, default=2048)
    parser.add_argument("--packing", action="store_true", help="Pack multiple short samples into fixed-length sequences.")
    parser.add_argument("--per-device-train-batch-size", type=int, default=1)
    parser.add_argument("--per-device-eval-batch-size", type=int, default=1)
    parser.add_argument("--gradient-accumulation-steps", type=int, default=16)
    parser.add_argument("--learning-rate", type=float, default=1e-4)
    parser.add_argument("--weight-decay", type=float, default=0.0)
    parser.add_argument(
        "--warmup-steps",
        type=float,
        default=0.03,
        help="Warmup steps as an absolute int or ratio float (<1.0).",
    )
    parser.add_argument("--max-grad-norm", type=float, default=1.0)
    parser.add_argument("--num-train-epochs", type=float, default=3.0)
    parser.add_argument("--lr-scheduler-type", type=str, default="cosine")
    parser.add_argument("--save-steps", type=int, default=10)
    parser.add_argument("--eval-steps", type=int, default=10)
    parser.add_argument("--logging-steps", type=int, default=10)
    parser.add_argument("--save-total-limit", type=int, default=3)
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--report-to", type=str, default="none", help='E.g. "none", "wandb", or "tensorboard".')
    parser.add_argument("--resume-from-checkpoint", type=str, default=None)

    return parser.parse_args()


def normalize_role(role: str) -> str:
    role = role.strip().lower()
    if role in {"human", "instruction"}:
        return "user"
    if role in {"bot", "model"}:
        return "assistant"
    return role


def as_message_list(value: Any, default_role: str) -> List[Dict[str, str]]:
    """
    Convert supported message encodings into a list of {"role": ..., "content": ...}.
    Accepts:
      - string
      - dict with role/content
      - list of dicts with role/content
    """
    if value is None:
        return []

    if isinstance(value, str):
        text = value.strip()
        return [{"role": default_role, "content": text}] if text else []

    if isinstance(value, dict):
        if "content" in value:
            role = normalize_role(str(value.get("role", default_role)))
            content = str(value["content"]).strip()
            return [{"role": role, "content": content}] if content else []
        raise ValueError(f"Unsupported dict message format: {value.keys()}")

    if isinstance(value, list):
        messages: List[Dict[str, str]] = []
        for item in value:
            if not isinstance(item, dict) or "content" not in item:
                raise ValueError(f"Unsupported list item in message list: {item!r}")
            role = normalize_role(str(item.get("role", default_role)))
            content = str(item["content"]).strip()
            if content:
                messages.append({"role": role, "content": content})
        return messages

    raise ValueError(f"Unsupported message value type: {type(value)}")


def split_messages_to_prompt_completion(messages: List[Dict[str, str]]) -> tuple[List[Dict[str, str]], List[Dict[str, str]]]:
    """
    If a row already contains a full conversation under `messages`, use the final assistant
    turn as completion and everything before it as prompt.
    """
    if not messages:
        raise ValueError("messages is empty")

    last_idx = None
    for idx in range(len(messages) - 1, -1, -1):
        if normalize_role(messages[idx]["role"]) == "assistant":
            last_idx = idx
            break

    if last_idx is None:
        raise ValueError("messages does not contain an assistant turn")
    if last_idx == 0:
        raise ValueError("messages contains only an assistant turn and no prompt")

    prompt = messages[:last_idx]
    completion = messages[last_idx:]
    return prompt, completion


def build_example(example: Dict[str, Any], user_field: str, assistant_field: str, system_prompt: str) -> Dict[str, Any]:
    # Preferred: prompt/completion already present.
    if "prompt" in example and "completion" in example:
        prompt = as_message_list(example["prompt"], default_role="user")
        completion = as_message_list(example["completion"], default_role="assistant")
    # Alternate: messages -> split last assistant message(s) as completion.
    elif "messages" in example:
        messages = as_message_list(example["messages"], default_role="user")
        prompt, completion = split_messages_to_prompt_completion(messages)
    # Simple custom fields: problem/solution, instruction/response, etc.
    elif user_field in example and assistant_field in example:
        prompt = [{"role": "user", "content": str(example[user_field]).strip()}]
        completion = [{"role": "assistant", "content": str(example[assistant_field]).strip()}]
    else:
        raise ValueError(
            f"Row must contain either prompt/completion, messages, or configured fields "
            f"{user_field!r}/{assistant_field!r}. Available keys: {list(example.keys())}"
        )

    if system_prompt:
        has_system = len(prompt) > 0 and normalize_role(prompt[0]["role"]) == "system"
        if not has_system:
            prompt = [{"role": "system", "content": system_prompt.strip()}] + prompt

    if not prompt:
        raise ValueError("Prompt is empty after preprocessing")
    if not completion:
        raise ValueError("Completion is empty after preprocessing")

    return {"prompt": prompt, "completion": completion}


def load_json_dataset(path: str, split_name: str = "train") -> Dataset:
    file_path = Path(path)
    if not file_path.exists():
        raise FileNotFoundError(f"Dataset file not found: {file_path}")

    suffix = file_path.suffix.lower()
    if suffix not in {".json", ".jsonl"}:
        raise ValueError(f"Unsupported file extension {suffix!r}. Use .json or .jsonl.")

    ds = load_dataset("json", data_files={split_name: str(file_path)})[split_name]
    return ds


def preprocess_row(
    example: Dict[str, Any],
    user_field: str,
    assistant_field: str,
    system_prompt: str,
) -> Dict[str, Any]:
    return build_example(example, user_field=user_field, assistant_field=assistant_field, system_prompt=system_prompt)


def preprocess_dataset(
    dataset: Dataset,
    user_field: str,
    assistant_field: str,
    system_prompt: str,
    tokenizer: Any,
    max_length: int,
    num_proc: Optional[int] = None,
) -> Dataset:
    original_columns = dataset.column_names
    processed = dataset.map(
        preprocess_row,
        fn_kwargs={
            "user_field": user_field,
            "assistant_field": assistant_field,
            "system_prompt": system_prompt,
        },
        remove_columns=original_columns,
        num_proc=num_proc,
        desc="Preprocessing dataset into conversational prompt-completion format",
    )
    max_tokens_with_buffer = max_length - 10
    processed = processed.filter(
        lambda example: len(
            tokenizer.apply_chat_template(
                [
                    {"role": str(msg.get("role", "")), "content": str(msg.get("content", ""))}
                    for msg in [*example.get("prompt", []), *example.get("completion", [])]
                ],
                tokenize=True,
                add_generation_prompt=False,
            )
        )
        <= max_tokens_with_buffer,
        desc="Dropping rows that exceed max sequence length",
    )
    return processed


def maybe_shuffle_dataset(dataset: Dataset, seed: int, enabled: bool) -> Dataset:
    return dataset.shuffle(seed=seed) if enabled else dataset


def apply_fim_transform(example: Dict[str, Any], rng: random.Random) -> Optional[Dict[str, Any]]:
    completion = example.get("completion", [])
    assistant_content = None
    for message in reversed(completion):
        role = normalize_role(str(message.get("role", "")))
        if role == "assistant":
            assistant_content = str(message.get("content", "")).strip()
            break

    if not assistant_content:
        return None

    tokens = assistant_content.split()
    if len(tokens) < 6:
        return None

    i = rng.randint(1, len(tokens) - 2)
    j = rng.randint(i + 1, len(tokens) - 1)

    prefix = " ".join(tokens[:i])
    middle = " ".join(tokens[i:j])
    suffix = " ".join(tokens[j:])
    fim_content = f"{FIM_PREFIX_TOKEN}{prefix}{FIM_SUFFIX_TOKEN}{suffix}{FIM_MIDDLE_TOKEN}{middle}"

    prompt = [
        {"role": str(msg.get("role", "")), "content": str(msg.get("content", ""))}
        for msg in example.get("prompt", [])
    ]
    return {
        "prompt": prompt,
        "completion": [{"role": "assistant", "content": fim_content}],
    }


def augment_with_fim(dataset: Dataset, fim_ratio: float, seed: int) -> Dataset:
    if fim_ratio <= 0.0:
        return dataset

    total_rows = count_rows(dataset)
    n_augment = math.ceil(total_rows * fim_ratio)
    if n_augment <= 0:
        return dataset

    rng = random.Random(seed)
    sampled_indices = rng.choices(range(total_rows), k=n_augment)

    fim_examples: List[Dict[str, Any]] = []
    for idx in sampled_indices:
        fim_example = apply_fim_transform(dataset[idx], rng)
        if fim_example is not None:
            fim_examples.append(fim_example)

    if not fim_examples:
        return dataset

    fim_dataset = Dataset.from_list(fim_examples)
    return concatenate_datasets([dataset, fim_dataset])


def train_val_split(dataset: Dataset, validation_ratio: float, seed: int) -> DatasetDict:
    if not (0.0 < validation_ratio < 1.0):
        raise ValueError("--validation-split-ratio must be between 0 and 1 when --eval-file is omitted.")
    split = dataset.train_test_split(test_size=validation_ratio, seed=seed, shuffle=True)
    return DatasetDict(train=split["train"], validation=split["test"])


def infer_precision(disable_bf16: bool) -> tuple[bool, bool, torch.dtype]:
    use_bf16 = (
        torch.cuda.is_available()
        and torch.cuda.is_bf16_supported()
        and not disable_bf16
    )
    use_fp16 = torch.cuda.is_available() and not use_bf16
    compute_dtype = torch.bfloat16 if use_bf16 else (torch.float16 if use_fp16 else torch.float32)
    return use_bf16, use_fp16, compute_dtype


def build_quant_config(use_4bit: bool, compute_dtype: torch.dtype) -> Optional[BitsAndBytesConfig]:
    if not use_4bit:
        return None

    return BitsAndBytesConfig(
        load_in_4bit=True,
        bnb_4bit_quant_type="nf4",
        bnb_4bit_use_double_quant=True,
        bnb_4bit_compute_dtype=compute_dtype,
    )


def get_target_modules(value: str) -> str | List[str]:
    if value.strip() == "all-linear":
        return "all-linear"
    return [item.strip() for item in value.split(",") if item.strip()]


def count_rows(dataset: Dataset) -> int:
    try:
        return dataset.num_rows
    except Exception:
        return len(dataset)


def sample_preview(dataset: Dataset, n: int = 1) -> List[Dict[str, Any]]:
    if count_rows(dataset) == 0:
        return []
    n = min(n, count_rows(dataset))
    idxs = random.sample(range(count_rows(dataset)), k=n)
    return [dataset[i] for i in idxs]


def write_run_config(args: argparse.Namespace, output_dir: str) -> None:
    os.makedirs(output_dir, exist_ok=True)
    with open(Path(output_dir) / "run_args.json", "w", encoding="utf-8") as f:
        json.dump(vars(args), f, indent=2, ensure_ascii=False)


def main() -> None:
    args = parse_args()
    os.makedirs(args.output_dir, exist_ok=True)
    set_seed(args.seed)
    write_run_config(args, args.output_dir)

    if torch.cuda.is_available():
        torch.backends.cuda.matmul.allow_tf32 = True
        torch.backends.cudnn.allow_tf32 = True

    # Load raw datasets.
    train_raw = load_json_dataset(args.train_file, split_name="train")

    if args.eval_file:
        eval_raw = load_json_dataset(args.eval_file, split_name="validation")
        datasets = DatasetDict(train=train_raw, validation=eval_raw)
    elif args.validation_split_ratio > 0:
        datasets = train_val_split(train_raw, validation_ratio=args.validation_split_ratio, seed=args.seed)
    else:
        datasets = DatasetDict(train=train_raw)

    # Tokenizer
    tokenizer = AutoTokenizer.from_pretrained(
        args.model_name,
        trust_remote_code=args.trust_remote_code,
        use_fast=True,
    )
    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token
    tokenizer.padding_side = "right"

    # Preprocess into conversational prompt-completion format.
    datasets["train"] = preprocess_dataset(
        datasets["train"],
        user_field=args.user_field,
        assistant_field=args.assistant_field,
        system_prompt=args.system_prompt,
        tokenizer=tokenizer,
        max_length=args.max_length,
        num_proc=args.dataset_num_proc,
    )
    datasets["train"] = maybe_shuffle_dataset(datasets["train"], seed=args.seed, enabled=args.shuffle_train)
    original_train_count = count_rows(datasets["train"])
    datasets["train"] = augment_with_fim(
        datasets["train"],
        fim_ratio=args.fim_augmentation_ratio,
        seed=args.seed,
    )
    augmented_train_count = count_rows(datasets["train"])
    fim_augmented_rows_added = augmented_train_count - original_train_count

    if "validation" in datasets:
        datasets["validation"] = preprocess_dataset(
            datasets["validation"],
            user_field=args.user_field,
            assistant_field=args.assistant_field,
            system_prompt=args.system_prompt,
            tokenizer=tokenizer,
            max_length=args.max_length,
            num_proc=args.dataset_num_proc,
        )

    print("=" * 80)
    print("Dataset summary")
    print(f"Train rows: {count_rows(datasets['train'])}")
    print(f"FIM-augmented rows added: {fim_augmented_rows_added}")
    if "validation" in datasets:
        print(f"Validation rows: {count_rows(datasets['validation'])}")
    print("=" * 80)

    # Precision / quantization
    use_bf16, use_fp16, compute_dtype = infer_precision(disable_bf16=args.disable_bf16)
    quant_config = build_quant_config(args.use_4bit, compute_dtype=compute_dtype)

    local_rank = int(os.environ.get("LOCAL_RANK", "0"))
    world_size = int(os.environ.get("WORLD_SIZE", "1"))
    device_map = {"": local_rank} if args.use_4bit and torch.cuda.is_available() else None

    model = AutoModelForCausalLM.from_pretrained(
        args.model_name,
        trust_remote_code=args.trust_remote_code,
        quantization_config=quant_config,
        torch_dtype=compute_dtype if not args.use_4bit else None,
        attn_implementation=args.attn_implementation,
        device_map=device_map,
    )

    model.config.use_cache = False

    if args.use_4bit:
        model = prepare_model_for_kbit_training(model)

    peft_config = LoraConfig(
        r=args.lora_r,
        lora_alpha=args.lora_alpha,
        lora_dropout=args.lora_dropout,
        bias="none",
        task_type="CAUSAL_LM",
        target_modules=get_target_modules(args.target_modules),
    )

    has_eval = "validation" in datasets

    training_args = SFTConfig(
        output_dir=args.output_dir,
        max_length=args.max_length,
        packing=args.packing,
        dataset_num_proc=args.dataset_num_proc,
        eos_token="<|im_end|>",
        pad_token=tokenizer.pad_token,
        per_device_train_batch_size=args.per_device_train_batch_size,
        per_device_eval_batch_size=args.per_device_eval_batch_size,
        gradient_accumulation_steps=args.gradient_accumulation_steps,
        learning_rate=args.learning_rate,
        weight_decay=args.weight_decay,
        warmup_steps=args.warmup_steps,
        max_grad_norm=args.max_grad_norm,
        num_train_epochs=args.num_train_epochs,
        lr_scheduler_type=args.lr_scheduler_type,
        logging_steps=args.logging_steps,
        save_steps=args.save_steps,
        eval_steps=args.eval_steps if has_eval else None,
        eval_strategy="steps" if has_eval else "no",
        save_strategy="steps",
        save_total_limit=args.save_total_limit,
        report_to=args.report_to,
        bf16=use_bf16,
        fp16=use_fp16,
        gradient_checkpointing=args.gradient_checkpointing,
        gradient_checkpointing_kwargs={"use_reentrant": False} if args.gradient_checkpointing else None,
        remove_unused_columns=True,
        completion_only_loss=True,
        assistant_only_loss=False,
        load_best_model_at_end=has_eval,
        metric_for_best_model="eval_loss" if has_eval else None,
        greater_is_better=False if has_eval else None,
        ddp_find_unused_parameters=False if world_size > 1 else None,
        seed=args.seed,
    )

    trainer = SFTTrainer(
        model=model,
        args=training_args,
        train_dataset=datasets["train"],
        eval_dataset=datasets["validation"] if has_eval else None,
        processing_class=tokenizer,
        peft_config=peft_config,
    )

    train_result = trainer.train(resume_from_checkpoint=args.resume_from_checkpoint)
    trainer.save_model(args.output_dir)
    tokenizer.save_pretrained(args.output_dir)

    metrics = train_result.metrics
    trainer.log_metrics("train", metrics)
    trainer.save_metrics("train", metrics)
    trainer.save_state()

    if has_eval:
        eval_metrics = trainer.evaluate()
        trainer.log_metrics("eval", eval_metrics)
        trainer.save_metrics("eval", eval_metrics)

    print("Training complete.")
    print(f"Saved adapter/tokenizer to: {args.output_dir}")


if __name__ == "__main__":
    main()
