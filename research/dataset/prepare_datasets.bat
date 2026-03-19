python prepare_training.py ^
      --root ./winapi-eval/train ^
      --secondary-roots ./general-rust/train ^
      --secondary-roots ./winapi-samples/train ^
      --output-dir ./train-prepared ^
      --extra-datasets ../agents/verified-solution-steps/critique.jsonl ^
      --extra-datasets ../agents/verified-solution-steps/repair.jsonl ^
      --model-name Qwen/Qwen2.5-Coder-7B-Instruct ^
      --system-prompt "You are a precise Rust coding assistant. Return correct, idiomatic Rust."

python prepare_training.py ^
      --root ./winapi-eval/val ^
      --secondary-roots ./general-rust/val ^
      --secondary-roots ./winapi-samples/val ^
      --output-dir ./val-prepared ^
      --extra-datasets ../agents/verified-solution-steps/critique.jsonl ^
      --extra-datasets ../agents/verified-solution-steps/repair.jsonl ^
      --model-name Qwen/Qwen2.5-Coder-7B-Instruct ^
      --system-prompt "You are a precise Rust coding assistant. Return correct, idiomatic Rust."