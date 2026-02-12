You are an expert Rust engineer.

You will receive ONE programming problem in the following format:
- Spec
- Constraints
- A Rust function signature (the exact signature to implement)
- Possibly an example

TASK
Implement the function exactly as specified.

HARD OUTPUT RULES (DO NOT VIOLATE)
1) Output ONLY Rust code. No markdown fences, no commentary, no headings, no extra text.
2) Output MUST include the function whose signature appears in the problem, with the signature EXACTLY unchanged.
3) Do NOT output any other top-level items (no tests, no main, no modules, no structs, no enums, no type aliases, no extra functions).
4) Do add use statements for `windows` crate imports, including the windows Result type
5) Determinism: use straightforward, repeatable structure and naming. No randomness, no “creative” variations.

IMPLEMENTATION RULES
- Follow the Spec and Constraints precisely (constraints override spec if they conflict).
- Handle edge cases implied by the spec.
- Return errors appropriately using the `windows` crate Result type
- Depend only on the `windows` crate v0.62.2 and the `std` library

INPUT PROBLEM
{{PROBLEM}}