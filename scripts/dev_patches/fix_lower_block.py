import os

with open('crates/apl_hir/src/lowering.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace(
'''    pub fn lower_block(&mut self, stmts: &[Stmt], span: Span) -> HirBlock {
        self.enter_scope();
        
        let mut hir_stmts = Vec::new();
        for stmt in stmts {
            if let Some(s) = self.lower_stmt(stmt) {
                hir_stmts.push(s);
            }
        }
        
        self.leave_scope();
        
        HirBlock {
            stmts: hir_stmts,
            span,
        }
    }''',
'''    pub fn lower_block(&mut self, stmts: &[Stmt], opt_expr: Option<&Box<apl_ast::Expr>>, span: Span) -> HirBlock {
        self.enter_scope();
        
        let mut hir_stmts = Vec::new();
        for stmt in stmts {
            if let Some(s) = self.lower_stmt(stmt) {
                hir_stmts.push(s);
            }
        }
        
        if let Some(e) = opt_expr {
            let h_e = self.lower_expr(e);
            hir_stmts.push(HirStmt::Return(Some(h_e), e.span));
        }
        
        self.leave_scope();
        
        HirBlock {
            stmts: hir_stmts,
            span,
        }
    }'''
)

text = text.replace(
'''        let body = if let ExprKind::Block(stmts, opt_expr) = &func.body.kind {
            let mut hir_block = self.lower_block(stmts, func.body.span);
            if let Some(e) = opt_expr {
                let h_e = self.lower_expr(e);
                hir_block.stmts.push(HirStmt::Return(Some(h_e), e.span));
            }
            Some(hir_block)
        } else {
            None
        };''',
'''        let body = if let ExprKind::Block(stmts, opt_expr) = &func.body.kind {
            Some(self.lower_block(stmts, opt_expr.as_ref(), func.body.span))
        } else {
            None
        };'''
)

# Fix Block(b, opt) in lower_expr
text = text.replace(
'''            ExprKind::Block(stmts, opt_expr) => {
                let h_block = self.lower_block(stmts, expr.span);''',
'''            ExprKind::Block(stmts, opt_expr) => {
                let h_block = self.lower_block(stmts, opt_expr.as_ref(), expr.span);'''
)

with open('crates/apl_hir/src/lowering.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("done")
