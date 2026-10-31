import os
import subprocess

tests = [
    {
        "name": "1. Frontend, Name Resolution & Shadowing",
        "desc": "Testing lexer limits (numbers, unicode), parser (nested structs), and scope shadowing. Ensures that AST and Name Resolution correctly map variables with identical names in nested scopes to distinct LocalIds.",
        "code": """struct Point {
    x: i32,
    y: i32
}

fn calculate_distance(a: i32, b: i32) -> i32 {
    let result = a * a + b * b;
    let temp = 1000;
    
    {
        let result = 42; // Shadowing inside nested scope
        let inner_temp = temp + result;
    }
    
    result // Should resolve to outer 'result' (LocalId 2)
}

fn main() -> i32 {
    let _shadow = 1;
    let _shadow = 2; // Shadowing in same scope
    calculate_distance(_shadow, 20)
}"""
    },
    {
        "name": "2. The Loop Maze (MIR & CFG Lowering)",
        "desc": "Massive control flow structure. Tests how MIR lowers nested `while`, `if`, `break`, and `continue` into a flat graph of basic blocks with `SwitchInt` and `Goto` terminators.",
        "code": """fn main() -> i32 {
    let state = 0;
    let counter = 100;
    
    while counter > 0 {
        if state == 10 {
            break;
        } else {
            if counter < 50 {
                state = state + 2;
                counter = counter - 2;
                continue;
            } else {
                state = state + 1;
            }
        }
        counter = counter - 1;
    }
    
    state
}"""
    },
    {
        "name": "3. The Temp Allocator (Expression Flattening)",
        "desc": "Tests how MIR flattens complex nested expressions (arithmetic, unary, binary, function calls) into primitive `Rvalue` operations and `Place` temporaries.",
        "code": """fn square(val: i32) -> i32 {
    val * val
}

fn main() -> i32 {
    let a = 5;
    let b = 10;
    let c = 15;
    
    let result = square(a) + square(b) * (c - a) / 2;
    
    if !(result > 100 && result < 1000) {
        result = 0;
    }
    
    result
}"""
    },
    {
        "name": "4. Dataflow Liveness & Initialization Tracking",
        "desc": "Tests the Dataflow Framework. Evaluates both backward Liveness (which variables are used later) and forward Initialization (which variables might be uninitialized).",
        "code": """fn main() -> i32 {
    let initialized = 10;
    let mut target = 0;
    let mut flag = true;
    
    if initialized > 5 {
        target = initialized;
    } else {
        target = 20;
    }
    
    while flag {
        if target > 100 {
            flag = false;
        }
        target = target + 1;
    }
    
    target
}"""
    }
]

out_md = "# YKL 0.4 - Full Compiler Pipeline Test Suite\n\n"
out_md += "This document contains a comprehensive set of tests validating the full semantic pipeline of the YKL compiler: Lexer/Parser, Name/Type Resolution (HIR), Control Flow Graph Lowering (MIR), and Iterative Dataflow Analysis (Liveness and Initialization).\n\n"

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
    out_md += "### Dataflow Analysis\n```text\n" + df_out.strip() + "\n```\n\n"
    
    out_md += "---\n\n"

with open(r'C:\Users\Али\.gemini\antigravity\brain\87f1a180-33ae-4036-8c8f-570f7ef0a9f7\massive_compiler_tests.md', 'w', encoding='utf-8') as f:
    f.write(out_md)

print("done")
