REM --existing-results ./out/winapi-eval/qwen-25-coder-base-quantized/run_report.json
REM https://openrouter.ai/api/v1

REM .\target\release\rust_eval_runner.exe --template ../benchmark_prompt.md --problems ../dataset/winapi-eval/benchmark-v2.json --model anthropic/claude-sonnet-4.6 --concurrency 4 --existing-results ./out/winapi-eval/claude-sonnet-46/run_report.json --openrouter-base https://openrouter.ai/api/v1

.\target\release\rust_eval_runner.exe --template ../simple_prompt.md --problems ../dataset/humaneval-rust/benchmark.json --model qwen3.5:9b --concurrency 4 --out-dir ./out/qwen-35-base_q4 --non-windows-deps
.\target\release\rust_eval_runner.exe --template ../simple_prompt.md --problems ../dataset/humaneval-rust/benchmark.json --model qwen2.5-coder:latest --concurrency 4 --out-dir ./out/qwen-coder25_q4 --non-windows-deps
.\target\release\rust_eval_runner.exe --template ../simple_prompt.md --problems ../dataset/humaneval-rust/benchmark.json --model qwen35-coder-rust-4bit:latest --concurrency 4 --out-dir ./out/winapi-coder-v2_q4 --non-windows-deps
.\target\release\rust_eval_runner.exe --template ../simple_prompt.md --problems ../dataset/humaneval-rust/benchmark.json --model qwen35-coder-rust-v2-4bit:latest --concurrency 4 --out-dir ./out/winapi-coder-v3_q4 --non-windows-deps