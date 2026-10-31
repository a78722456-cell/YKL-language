import os

with open('crates/apl_hir/src/lib.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace(
'''    If {
        cond: Box<HirExpr>,
        then_branch: HirBlock,
        else_branch: Option<HirBlock>,
    },''',
'''    If {
        cond: Box<HirExpr>,
        then_branch: Box<HirExpr>,
        else_branch: Option<Box<HirExpr>>,
    },'''
)
text = text.replace(
'''    While {
        cond: Box<HirExpr>,
        body: HirBlock,
    },''',
'''    While {
        cond: Box<HirExpr>,
        body: Box<HirExpr>,
    },'''
)
text = text.replace(
'''    Block(Vec<Stmt>, Option<Box<Expr>>),''', # not here
''
)
# Add Block to HirExprKind
text = text.replace(
'''    While {
        cond: Box<HirExpr>,
        body: Box<HirExpr>,
    },''',
'''    While {
        cond: Box<HirExpr>,
        body: Box<HirExpr>,
    },
    Block(HirBlock, Option<Box<HirExpr>>),'''
)

with open('crates/apl_hir/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print('done')
