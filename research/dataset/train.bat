REM conda activate qwen-rust-ft

set TENSORBOARD_LOGGING_DIR=outputs/tb

accelerate launch train_qwen35.py ^
  --train-file train-prepared/dataset.jsonl ^
  --eval-file val-prepared/dataset.jsonl  ^
  --output-dir outputs/qwen35-rust-winapi-qlora-v2.1 ^
  --model-name Qwen/Qwen3.5-9B-Base ^
  --max-length 2048 ^
  --per-device-train-batch-size 1 ^
  --gradient-accumulation-steps 16 ^
  --learning-rate 1e-4 ^
  --num-train-epochs 3 ^
  --use-4bit ^
  --report-to tensorboard