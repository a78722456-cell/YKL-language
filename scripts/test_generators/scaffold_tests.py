import os

dirs = [
    "lexer", "parser", "ast", "resolution", "types", "generics", "traits", "adt",
    "patterns", "expressions", "statements", "ownership", "borrowing", "lifetimes",
    "initialization", "control_flow", "hir", "mir", "cfg", "dataflow", "drops",
    "codegen", "vm", "differential", "ffi", "abi", "cli", "diagnostics", "std",
    "integration", "compile-pass", "compile-fail", "fuzz", "property", "regression",
    "performance", "security"
]

for d in dirs:
    os.makedirs(os.path.join("tests", d), exist_ok=True)
    with open(os.path.join("tests", d, ".gitkeep"), 'w') as f:
        pass

print("done")
