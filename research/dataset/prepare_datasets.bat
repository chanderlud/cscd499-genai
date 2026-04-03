python prepare_training.py ^
      --root ./winapi-eval/train ^
      --secondary-roots ./general-rust/train ^
      --secondary-roots ./winapi-samples/train ^
      --secondary-roots ./winapi-usage/train ^
      --output-dir ./train-prepared ^
      --extra-datasets ../agents/verified-solution-steps/critique.jsonl ^
      --extra-datasets ../agents/verified-solution-steps/repair.jsonl ^
      --conversational-datasets personal-rust\personal_rust_custom_train.jsonl ^
      --model-name Qwen/Qwen3.5-9B-Base ^
      --system-prompt "You are a Rust code generator. Output ONLY valid Rust source code." ^
      --repair-system-prompt "You are a Rust code repair generator, especially for Windows API code using the windows crate. You will receive a problem specification, broken Rust code, and feedback such as compiler errors or test failures. Produce a corrected, complete Rust implementation that fixes the reported issues and satisfies the specification. Output only valid Rust source code, with no markdown or commentary. Keep the solution minimal and use only the imports and Windows API items actually needed." ^
      --critique-system-prompt "You are a Rust code reviewer for Windows API code using the windows crate. You will receive a problem specification, candidate Rust code, and feedback such as compiler errors or test failures. Return exactly one JSON object with these fields: ""bug_category"", ""diagnosis"", ""repair_plan"", and ""key_insight"". The diagnosis should explain the bug and why the feedback occurred, the repair plan should give concrete fix steps, and the key insight should capture the main lesson. Output only the JSON object, with no markdown or extra text."

python prepare_training.py ^
      --root ./winapi-eval/val ^
      --secondary-roots ./general-rust/val ^
      --secondary-roots ./winapi-samples/val ^
      --secondary-roots ./winapi-usage/val ^
      --output-dir ./val-prepared ^
      --extra-datasets ../agents/verified-solution-steps/critique.jsonl ^
      --extra-datasets ../agents/verified-solution-steps/repair.jsonl ^
      --conversational-datasets personal-rust\personal_rust_custom_train.jsonl ^
      --model-name Qwen/Qwen3.5-9B-Base ^
      --system-prompt "You are a Rust code generator. Output ONLY valid Rust source code." ^
      --repair-system-prompt "You are a Rust code repair generator, especially for Windows API code using the windows crate. You will receive a problem specification, broken Rust code, and feedback such as compiler errors or test failures. Produce a corrected, complete Rust implementation that fixes the reported issues and satisfies the specification. Output only valid Rust source code, with no markdown or commentary. Keep the solution minimal and use only the imports and Windows API items actually needed." ^
      --critique-system-prompt "You are a Rust code reviewer for Windows API code using the windows crate. You will receive a problem specification, candidate Rust code, and feedback such as compiler errors or test failures. Return exactly one JSON object with these fields: ""bug_category"", ""diagnosis"", ""repair_plan"", and ""key_insight"". The diagnosis should explain the bug and why the feedback occurred, the repair plan should give concrete fix steps, and the key insight should capture the main lesson. Output only the JSON object, with no markdown or extra text."
