from uuid import uuid4

problems_str = open("staging.md", "r", errors="ignore").read()
problems = problems_str.split("\n---\n\n")

for problem in problems:
    with open(f"staging/problems/{uuid4()}.md", "w+") as f:
        f.write(problem)
