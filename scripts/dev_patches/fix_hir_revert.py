import os

with open('crates/apl_hir/src/lib.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace(
'''    pub return_type: Type,
    pub body: Option<HirExpr>, // None if extern
    pub is_extern: bool,''',
'''    pub return_type: Type,
    pub body: Option<HirBlock>, // None if extern
    pub is_extern: bool,'''
)

with open('crates/apl_hir/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(text)

with open('crates/apl_hir/src/lowering.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace(
'''        let body = Some(self.lower_expr(&func.body));''',
'''        let body = if let ExprKind::Block(stmts, opt_expr) = &func.body.kind {
            let mut hir_block = self.lower_block(stmts, func.body.span);
            if let Some(e) = opt_expr {
                let h_e = self.lower_expr(e);
                hir_block.stmts.push(HirStmt::Return(Some(h_e), e.span));
            }
            Some(hir_block)
        } else {
            None
        };'''
)

with open('crates/apl_hir/src/lowering.rs', 'w', encoding='utf-8') as f:
    f.write(text)

with open('crates/apl_mir/src/lowering.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace(
'''        if let Some(body) = &func.body {
            let rval = self.lower_expr(body);
            self.blocks[self.current_block.0 as usize].statements.push(Statement::Assign(Place::Local(LocalId(0)), rval, func.span));
        }
        self.blocks[self.current_block.0 as usize].terminator = Terminator::Return(func.span);''',
'''        if let Some(body) = &func.body {
            self.lower_block(body);
        }
        self.blocks[self.current_block.0 as usize].terminator = Terminator::Return(func.span);'''
)

with open('crates/apl_mir/src/lowering.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("done")
