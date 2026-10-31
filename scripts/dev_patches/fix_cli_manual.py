import os

with open('crates/apl_cli/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# We need to find the let mut sema = apl_sema::Sema::new(); line and strip everything after it inside that function.
idx = text.find('let mut sema = apl_sema::Sema::new();')
if idx != -1:
    end_idx = text.find('}\n\nfn run_tests', idx)
    if end_idx != -1:
        text = text[:idx] + 'return;\n    ' + text[end_idx:]

with open('crates/apl_cli/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print("done")
