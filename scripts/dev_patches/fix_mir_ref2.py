import re

with open('crates/apl_mir/src/lowering.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace('Rvalue::Ref((),', 'Rvalue::Ref(crate::Region(0),')

with open('crates/apl_mir/src/lowering.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("done")
