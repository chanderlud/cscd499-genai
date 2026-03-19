REM conda activate qwen-rust-ft

accelerate launch train.py ^
  --train-file train-prepared/dataset.jsonl ^
  --eval-file val-prepared/dataset.jsonl  ^
  --output-dir outputs/qwen25-coder-rust-7b-qlora ^
  --model-name Qwen/Qwen2.5-Coder-7B-Instruct ^
  --max-length 4096 ^
  --per-device-train-batch-size 1 ^
  --gradient-accumulation-steps 16 ^
  --learning-rate 2e-5 ^
  --num-train-epochs 16 ^
  --use-4bit