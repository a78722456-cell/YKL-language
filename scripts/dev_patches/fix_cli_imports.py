import os

with open('crates/apl_cli/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace('use apl_codegen::CCodeGenerator;', '// use apl_codegen::CCodeGenerator;')
text = text.replace('use apl_mir::{MirBuilder, MirValue};', '// use apl_mir::{MirBuilder, MirValue};')
text = text.replace('use apl_opt::MirOptimizer;', '// use apl_opt::MirOptimizer;')
text = text.replace('use apl_vm::VirtualMachine;', '// use apl_vm::VirtualMachine;')

with open('crates/apl_cli/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print("done")
