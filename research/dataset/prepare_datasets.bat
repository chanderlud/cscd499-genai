python prepare_training.py ^
      --root ./winapi-eval/train ^
      --secondary-roots ./general-rust/train ^
      --secondary-roots ./winapi-samples/train ^
      --secondary-roots ./winapi-usage/train ^
      --output-dir ./train-prepared ^
      --extra-datasets ../agents/verified-solution-steps/critique.jsonl ^
      --extra-datasets ../agents/verified-solution-steps/repair.jsonl ^
      --model-name Qwen/Qwen3.5-9B-Base ^
      --system-prompt "You are a precise Rust coding assistant. Return correct, idiomatic Rust."

python prepare_training.py ^
      --root ./winapi-eval/val ^
      --secondary-roots ./general-rust/val ^
      --secondary-roots ./winapi-samples/val ^
      --secondary-roots ./winapi-usage/val ^
      --output-dir ./val-prepared ^
      --extra-datasets ../agents/verified-solution-steps/critique.jsonl ^
      --extra-datasets ../agents/verified-solution-steps/repair.jsonl ^
      --model-name Qwen/Qwen3.5-9B-Base ^
      --system-prompt "You are a precise Rust coding assistant. Return correct, idiomatic Rust."