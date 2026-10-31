import re

with open('crates/apl_mir/src/lib.rs', 'r', encoding='utf-8') as f:
    text = f.read()

replacement = '''pub enum Rvalue {
    Use(Operand),
    Ref(Region, BorrowKind, Place),
    BinaryOp(apl_ast::BinaryOp, Operand, Operand),
    UnaryOp(apl_ast::UnaryOp, Operand),
    Aggregate(Vec<Operand>),
}'''

text = text.replace('''pub enum Rvalue {
    Use(Operand),
    Ref(Region, BorrowKind, Place),
    BinaryOp(apl_ast::BinaryOp, Operand, Operand),
    UnaryOp(apl_ast::UnaryOp, Operand),
}''', replacement)

with open('crates/apl_mir/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(text)
