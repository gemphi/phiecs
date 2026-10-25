# A stand-in oracle for `persona_quest` (run without ollama): reads the prompt
# on stdin, answers the relation asked with three words drawn deterministically
# from the word, in the exact one-kind format. Not a model; a fixed pool.
import sys, re, hashlib
p = sys.stdin.read()
w = re.search(r"^Word: (.+)$", p, re.M)
k = re.search(r"^Relation: (.+)$", p, re.M)
pool = "water fish boat river sea net wave stone shore wind sail rope salt".split()
if not (w and k):
    print("The sea remembers every boat that crossed it.")  # a voice answer
    sys.exit(0)
word, kind = w.group(1).strip(), k.group(1).strip()
h = int(hashlib.md5((word + kind).encode()).hexdigest(), 16)
others = [x for x in pool if x != word]
for i in range(3):
    o = others[(h >> (4 * i)) % len(others)]
    print(f"{kind} | {o} | 0.{5 + i}")
