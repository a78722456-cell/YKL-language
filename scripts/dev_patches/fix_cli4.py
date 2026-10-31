import re
with open('crates/apl_cli/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace('if dump_mir || dump_cfg || dump_dataflow {', 'if dump_mir || dump_cfg || dump_dataflow || dump_borrowck {')

with open('crates/apl_cli/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("done")
