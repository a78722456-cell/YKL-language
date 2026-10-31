import os

with open('crates/apl_cli/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# Fix compile_workspace signature
text = text.replace('fn compile_workspace(_run_vm: bool, input_file: Option<&str>, dump_hir: bool, dump_mir: bool, dump_cfg: bool, dump_dataflow: bool) -> anyhow::Result<()> {',
'fn compile_workspace(_run_vm: bool, input_file: Option<&str>, dump_hir: bool, dump_mir: bool, dump_cfg: bool, dump_dataflow: bool, dump_borrowck: bool) -> anyhow::Result<()> {')

# Find the bad block in Fmt
start_bad = text.find('if dump_borrowck {')
end_bad = text.find('println!("{} code formatted."', start_bad)
if start_bad != -1 and end_bad != -1:
    text = text[:start_bad] + text[end_bad-17:]

# Remove duplicate exit
text = text.replace('std::process::exit(0);\n            std::process::exit(0);', 'std::process::exit(0);')

with open('crates/apl_cli/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("done")
