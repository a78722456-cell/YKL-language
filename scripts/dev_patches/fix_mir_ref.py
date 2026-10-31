import re

with open('crates/apl_mir/src/lowering.rs', 'r', encoding='utf-8') as f:
    text = f.read()

replacement = '''            HirExprKind::UnaryOp(op, inner) => {
                match op {
                    apl_ast::UnaryOp::Ref => {
                        let place = self.lower_place(inner);
                        Rvalue::Ref((), BorrowKind::Shared, place)
                    }
                    apl_ast::UnaryOp::RefMut => {
                        let place = self.lower_place(inner);
                        Rvalue::Ref((), BorrowKind::Mut, place)
                    }
                    _ => {
                        let inner_rv = self.lower_expr(inner);
                        let temp = self.make_temp(inner_rv, inner.span, inner.ty.clone());
                        Rvalue::UnaryOp(op.clone(), Operand::Copy(temp))
                    }
                }
            }'''

text = re.sub(
    r'HirExprKind::UnaryOp\(op, inner\) => \{\s*let inner_rv = self\.lower_expr\(inner\);\s*let temp = self\.make_temp\(inner_rv, inner\.span, inner\.ty\.clone\(\)\);\s*Rvalue::UnaryOp\(op\.clone\(\), Operand::Copy\(temp\)\)\s*\}',
    replacement,
    text
)

with open('crates/apl_mir/src/lowering.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print("done")
