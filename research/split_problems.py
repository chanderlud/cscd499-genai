from uuid import uuid4

problems_str = open("staging.md", "r").read()
problems = problems_str.split("\n\n### ")

for problem in problems:
    problem_str = problem.replace("### ", "")
    with open(f"problems/{uuid4()}.md", "w+") as f:
        f.write(problem_str)
