import re
with open('crates/apl_mir/src/lowering.rs', 'r', encoding='utf-8') as f:
    text = f.read()

replacement = '''            HirExprKind::LocalRef(id) => {
                if expr.ty.is_copy() {
                    Rvalue::Use(Operand::Copy(Place::Local(*id)))
                } else {
                    Rvalue::Use(Operand::Move(Place::Local(*id)))
                }
            }'''

text = re.sub(r'HirExprKind::LocalRef\(id\) => Rvalue::Use\(Operand::Copy\(Place::Local\(\*id\)\)\),', replacement, text)

with open('crates/apl_mir/src/lowering.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print("done")
