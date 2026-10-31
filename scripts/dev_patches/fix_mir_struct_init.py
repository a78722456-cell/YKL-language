import re

with open('crates/apl_mir/src/lowering.rs', 'r', encoding='utf-8') as f:
    text = f.read()

replacement = '''            HirExprKind::StructInit { struct_id: _, fields } => {
                let mut ops = Vec::new();
                for (_, field_expr) in fields {
                    let f_rv = self.lower_expr(field_expr);
                    let temp = self.make_temp(f_rv, field_expr.span, field_expr.ty.clone());
                    ops.push(Operand::Copy(temp));
                }
                Rvalue::Aggregate(ops)
            }
            _ => Rvalue::Use(Operand::Constant(Constant::Int(0, Type::Void))),'''

text = text.replace('_ => Rvalue::Use(Operand::Constant(Constant::Int(0, Type::Void))),', replacement)

with open('crates/apl_mir/src/lowering.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("done")
