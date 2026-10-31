import os
import re

with open('crates/apl_cli/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# Fix the Commands::Run pattern
text = re.sub(
    r'Commands::Run \{ release, input, dump_hir, dump_mir, dump_cfg, dump_dataflow \} => \{',
    'Commands::Run { release, input, dump_hir, dump_mir, dump_cfg, dump_dataflow, dump_borrowck } => {',
    text
)

# Fix the compile_workspace definition
text = re.sub(
    r'fn compile_workspace\(_run_vm: bool, input_file: Option<&str>, dump_hir: bool, dump_mir: bool, dump_cfg: bool, dump_dataflow: bool\) -> anyhow::Result<\(\)> \{',
    'fn compile_workspace(_run_vm: bool, input_file: Option<&str>, dump_hir: bool, dump_mir: bool, dump_cfg: bool, dump_dataflow: bool, dump_borrowck: bool) -> anyhow::Result<()> {',
    text
)

# Remove the bad replacement I did that injected broken code
# The bad block starts with "if dump_borrowck {" around line 145 which is outside the function!
# Wait, I can just restore from git?
