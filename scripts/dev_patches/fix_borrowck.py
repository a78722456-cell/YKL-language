import os

with open('crates/apl_borrowck/src/lib.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace('use apl_hir::Span;', 'use apl_ast::Span;')
text = text.replace('apl_mir::LocalId', 'apl_hir::LocalId')

with open('crates/apl_borrowck/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(text)

with open('crates/apl_borrowck/src/points_to.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace('use apl_hir::Span;', 'use apl_ast::Span;')
text = text.replace('apl_mir::{MirBody, LocalId', 'apl_mir::{MirBody, Statement')
text = text.replace('use apl_mir::dataflow::DataflowAnalysis;', 'use apl_mir::dataflow::DataflowAnalysis;\nuse apl_hir::LocalId;')

with open('crates/apl_borrowck/src/points_to.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("done")
