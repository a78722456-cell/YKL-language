import os
import subprocess
import json

tests = [
    {
        "name": "1. Complex Control Flow (Loops & Jumps)",
        "desc": "Testing nested structures, unconditional jumps, and temporary evaluation.",
        "code": """fn main() -> i32 {
    let x = 0;
    let y = 100;
    while x < 50 {
        if x == 25 {
            break;
        } else {
            if y < 10 {
                x = x + 2;
                continue;
            } else {
                x = x + 1;
            }
        }
        y = y - 1;
    }
    x + y
}"""
    },
    {
        "name": "2. Temporary Variables & Function Calls",
        "desc": "Testing how MIR flattens complex expressions into temporary variables (StorageLive/Dead).",
        "code": """fn calculate(a: i32, b: i32) -> i32 {
    a * b + (a - b)
}

fn main() -> i32 {
    let result = calculate(10, 20) + calculate(5, 5);
    if result > 100 {
        result
    } else {
        0
    }
}"""
    },
    {
        "name": "3. Dataflow Liveness Tracking",
        "desc": "Testing how the Dataflow framework calculates LiveIn and LiveOut sets across branches.",
        "code": """fn main() -> i32 {
    let a = 1;
    let b = 2;
    let c = 3;
    let result = 0;
    
    if a < b {
        result = a;
    } else {
        result = c;
    }
    
    result
}"""
    }
]

out_md = "# YKL 0.4 Compiler Pipeline Test Results\n\n"
out_md += "This document contains the automated test results for the YKL Semantic Compiler Core (HIR, Expression-less MIR, and Dataflow Analysis).\n\n"

for i, t in enumerate(tests):
    with open('test.yk', 'w', encoding='utf-8') as f:
        f.write(t["code"])
        
    out_md += f"## {t['name']}\n"
    out_md += f"{t['desc']}\n\n"
    out_md += "### Source Code (YKL)\n```rust\n" + t["code"] + "\n```\n\n"
    
    # Run HIR
    res = subprocess.run(['cargo', 'run', '-p', 'apl_cli', '--', 'run', '--dump-hir'], capture_output=True, text=True, encoding='utf-8')
    hir_out = res.stdout
    hir_out = "\n".join([line for line in hir_out.split('\n') if not line.startswith('--> running')])
    out_md += "### HIR (High-Level Intermediate Representation)\n```rust\n" + hir_out.strip() + "\n```\n\n"

    # Run MIR
    res = subprocess.run(['cargo', 'run', '-p', 'apl_cli', '--', 'run', '--dump-mir'], capture_output=True, text=True, encoding='utf-8')
    mir_out = res.stdout
    mir_out = "\n".join([line for line in mir_out.split('\n') if not line.startswith('--> running')])
    out_md += "### MIR (Mid-Level Intermediate Representation)\n```rust\n" + mir_out.strip() + "\n```\n\n"
    
    # Run Dataflow
    res = subprocess.run(['cargo', 'run', '-p', 'apl_cli', '--', 'run', '--dump-dataflow'], capture_output=True, text=True, encoding='utf-8')
    df_out = res.stdout
    df_out = "\n".join([line for line in df_out.split('\n') if not line.startswith('--> running')])
    out_md += "### Dataflow Analysis (Liveness)\n```text\n" + df_out.strip() + "\n```\n\n"
    
    out_md += "---\n\n"

with open(r'C:\Users\Али\.gemini\antigravity\brain\87f1a180-33ae-4036-8c8f-570f7ef0a9f7\compiler_pipeline_tests.md', 'w', encoding='utf-8') as f:
    f.write(out_md)

print("done")
