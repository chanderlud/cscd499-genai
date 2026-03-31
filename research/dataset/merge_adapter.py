from transformers import AutoModelForCausalLM, AutoTokenizer
from peft import PeftModel
import torch

base_model = "Qwen/Qwen3.5-9B-Base"
adapter_dir = r"outputs/qwen35-rust-winapi-qlora-v2.1/checkpoint-150"
merged_dir = r"merged_qwen35_coder_rust_9b_v2.1"

tokenizer = AutoTokenizer.from_pretrained(base_model)

model = AutoModelForCausalLM.from_pretrained(
    base_model,
    torch_dtype=torch.float16,
    device_map="auto",
)

model = PeftModel.from_pretrained(model, adapter_dir)
model = model.merge_and_unload()

model.save_pretrained(merged_dir, safe_serialization=True)
tokenizer.save_pretrained(merged_dir)

print(f"Merged model saved to: {merged_dir}")