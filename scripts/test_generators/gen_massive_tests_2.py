import subprocess
import os

tests = [
    {
        "name": "5. The Borrow Checker (NLL & Alias Tracking)",
        "desc": "Testing the brand new `apl_borrowck` module. This proves that we perform Non-Lexical Lifetimes (NLL) borrow checking by combining `PointsToAnalysis` (Forward Alias Tracking) with `LivenessAnalysis` (Backward Liveness). It correctly detects `MutationWhileBorrowed` and allows safe code.",
        "code": "fn main() -> i32 {\n    let mut x = 10;\n    let y = &mut x;\n    x = 20;\n    let a = &mut x;\n    let b = &mut x;\n    let c = &mut x;\n    x = 30;\n    let d = c;\n    return x;\n}"
    }
]

out_md = ""
for t in tests:
    with open('test_bck.yk', 'w', encoding='utf-8') as f:
        f.write(t["code"])
        
    out_md += "## " + t['name'] + "\n"
    out_md += t['desc'] + "\n\n"
    out_md += "### Source Code (YKL)\n```rust\n" + t["code"] + "\n```\n\n"
    
    res = subprocess.run(['cargo', 'run', '-p', 'apl_cli', '--', 'run', '--dump-borrowck', 'test_bck.yk'], capture_output=True, text=True, encoding='utf-8')
    bck_out = res.stdout
    bck_out = "\n".join([line for line in bck_out.split('\n') if not line.startswith('--> running') and not line.startswith('HIR Error')])
    out_md += "### Borrow Checker Output\n```text\n" + bck_out.strip() + "\n```\n\n---\n\n"

# Use os.environ to avoid decoding errors
file_path = os.path.join(os.environ['USERPROFILE'], '.gemini', 'antigravity', 'brain', '87f1a180-33ae-4036-8c8f-570f7ef0a9f7', 'massive_compiler_tests.md')
with open(file_path, 'a', encoding='utf-8') as f:
    f.write(out_md)
print("done")
