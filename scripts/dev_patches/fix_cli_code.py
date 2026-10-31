import os

with open('crates/apl_cli/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()

start_idx = text.find('let mut mir_funcs = Vec::new();')
if start_idx != -1:
    end_idx = text.find('}\n\nfn parse_command_line', start_idx)
    if end_idx != -1:
        text = text[:start_idx] + 'return;\n    ' + text[end_idx:]

with open('crates/apl_cli/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print("done")
