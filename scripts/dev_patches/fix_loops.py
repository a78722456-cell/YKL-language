import os

with open('crates/apl_mir/src/lowering.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace(
'''pub struct MirBuilder {
    locals: Vec<LocalDecl>,
    blocks: Vec<BasicBlock>,
    current_block: BasicBlockId,
}''',
'''pub struct MirBuilder {
    locals: Vec<LocalDecl>,
    blocks: Vec<BasicBlock>,
    current_block: BasicBlockId,
    loop_scopes: Vec<(BasicBlockId, BasicBlockId)>,
}'''
)

text = text.replace(
'''            locals: Vec::new(),
            blocks: Vec::new(),
            current_block: BasicBlockId(0),
        };''',
'''            locals: Vec::new(),
            blocks: Vec::new(),
            current_block: BasicBlockId(0),
            loop_scopes: Vec::new(),
        };'''
)

# In lower_expr, handle break and continue (Wait, Break and Continue are not in HirExprKind! They are not currently in HirExprKind! Wait, does apl_hir even have break/continue?)
