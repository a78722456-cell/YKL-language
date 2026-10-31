import os

with open('crates/apl_cli/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# find let mut sema = SemanticAnalyzer::new();
idx = text.find('let mut mir_funcs = Vec::new();')
if idx != -1:
    end_idx = text.find('    Ok(())\n}', idx)
    if end_idx != -1:
        text = text[:idx] + 'return Ok(());\n' + text[end_idx:]

with open('crates/apl_cli/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print("done")
