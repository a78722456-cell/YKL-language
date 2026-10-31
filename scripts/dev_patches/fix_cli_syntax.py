import re

with open('crates/apl_cli/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# Fix the broken println
broken_str = '''                        if dump_cfg {
                            println!("--- CFG for {} ---", func.name);
                            println!("{}
                        if dump_dataflow {'''
fixed_str = '''                        if dump_cfg {
                            println!("--- CFG for {} ---", func.name);
                            println!("{}", apl_mir::printer::dump_cfg_dot(&mir_body, &func.name));
                        }
                        if dump_dataflow {'''

text = text.replace(broken_str, fixed_str)

# Fix the missing param 6
text = text.replace('compile_workspace(true, input.as_deref(), dump_hir, dump_mir, dump_cfg)?;', 'compile_workspace(true, input.as_deref(), *dump_hir, *dump_mir, *dump_cfg, *dump_dataflow)?;')

with open('crates/apl_cli/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print("done")
