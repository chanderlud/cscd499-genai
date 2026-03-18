from transformers import AutoModelForCausalLM, AutoTokenizer
from peft import PeftModel
import torch

base_model = "Qwen/Qwen2.5-Coder-7B-Instruct"
adapter_dir = r"outputs/qwen25-coder-rust-7b-qlora"
merged_dir = r"merged_qwen25_coder_rust_7b"

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