import re

# Fix printer.rs
with open('crates/apl_mir/src/printer.rs', 'r', encoding='utf-8') as f:
    text = f.read()
text = text.replace('Rvalue::UnaryOp(op, val) => format!("{:?}{}", op, dump_operand(val)),',
                    'Rvalue::UnaryOp(op, val) => format!("{:?}{}", op, dump_operand(val)),\n        Rvalue::Aggregate(ops) => format!("{{ {} }}", ops.iter().map(|o| dump_operand(o)).collect::<Vec<_>>().join(", ")),')
with open('crates/apl_mir/src/printer.rs', 'w', encoding='utf-8') as f:
    f.write(text)

# Fix validate.rs
with open('crates/apl_mir/src/validate.rs', 'r', encoding='utf-8') as f:
    text = f.read()
text = text.replace('Rvalue::UnaryOp(_, op) => self.validate_operand(op, local_ids)?,',
                    'Rvalue::UnaryOp(_, op) => self.validate_operand(op, local_ids)?,\n            Rvalue::Aggregate(ops) => { for op in ops { self.validate_operand(op, local_ids)?; } },')
with open('crates/apl_mir/src/validate.rs', 'w', encoding='utf-8') as f:
    f.write(text)

# Fix apl_borrowck
with open('crates/apl_borrowck/src/lib.rs', 'r', encoding='utf-8') as f:
    text = f.read()
text = text.replace('Rvalue::BinaryOp(_, l, r) => { check_op(l); check_op(r); }',
                    'Rvalue::BinaryOp(_, l, r) => { check_op(l); check_op(r); }\n        Rvalue::Aggregate(ops) => { for op in ops { check_op(op); } },')
with open('crates/apl_borrowck/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("done")
