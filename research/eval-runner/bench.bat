REM --existing-results ./out/winapi-eval/qwen-25-coder-base-quantized/run_report.json
REM https://openrouter.ai/api/v1

REM .\target\release\rust_eval_runner.exe --template ../benchmark_prompt.md --problems ../dataset/winapi-eval/benchmark-v2.json --model anthropic/claude-sonnet-4.6 --concurrency 4 --existing-results ./out/winapi-eval/claude-sonnet-46/run_report.json --openrouter-base https://openrouter.ai/api/v1

.\target\release\rust_eval_runner.exe --template ../simple_prompt.md --problems ../dataset/humaneval-rust/benchmark.json --model openai/gpt-5.4-mini --concurrency 4 --openrouter-base https://openrouter.ai/api/v1