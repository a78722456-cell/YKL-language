import os

with open('crates/apl_hir/src/lowering.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace(
'''    pub fn lower_block(&mut self, stmts: &[apl_ast::Stmt], span: Span) -> HirBlock {
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
'''    pub fn lower_block(&mut self, stmts: &[apl_ast::Stmt], opt_expr: Option<&Box<apl_ast::Expr>>, span: Span) -> HirBlock {
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

with open('crates/apl_hir/src/lowering.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print("done")
