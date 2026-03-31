REM --existing-results ./out/winapi-eval/qwen-25-coder-base-quantized/run_report.json

.\target\release\rust_eval_runner.exe --template ../benchmark_prompt.md --problems ../dataset/winapi-eval/benchmark-v2.json --model qwen35-coder-rust-v2-4bit:latest --concurrency 4