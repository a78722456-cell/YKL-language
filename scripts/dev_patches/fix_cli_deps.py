import os

with open('crates/apl_cli/Cargo.toml', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace('apl_vm =', '# apl_vm =')
text = text.replace('apl_opt =', '# apl_opt =')
text = text.replace('apl_codegen =', '# apl_codegen =')
text = text.replace('apl_mir = { path = "../apl_mir" }', 'apl_mir = { path = "../apl_mir" }\napl_hir = { path = "../apl_hir" }')

with open('crates/apl_cli/Cargo.toml', 'w', encoding='utf-8') as f:
    f.write(text)

with open('crates/apl_cli/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# Comment out imports
text = text.replace('use apl_vm::Vm;', '// use apl_vm::Vm;')
text = text.replace('use apl_opt::optimize;', '// use apl_opt::optimize;')

# Comment out the old execution code since it uses vm and opt
# We will just insert return after Sema for now unless dump flags are passed.
parse_idx = text.find('let mut sema = apl_sema::Sema::new();')
if parse_idx != -1:
    old_code_start = text.find('let mir = apl_mir::build_mir(&program);', parse_idx)
    if old_code_start != -1:
        text = text[:old_code_start] + '    // Old MIR and VM disabled\n    return;\n' + text[old_code_start:].replace('\n', '\n// ')

with open('crates/apl_cli/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print("done")
