import re

with open('crates/apl_hir/src/lowering.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = re.sub(
    r'pub fn lower_block\(&mut self, stmts: &\[apl_ast::Stmt\], span: Span\) -> HirBlock \{\s*self\.enter_scope\(\);\s*let mut hir_stmts = Vec::new\(\);',
    '''pub fn lower_block(&mut self, stmts: &[apl_ast::Stmt], opt_expr: Option<&Box<apl_ast::Expr>>, span: Span) -> HirBlock {
        self.enter_scope();
        let mut hir_stmts = Vec::new();''',
    text
)

text = re.sub(
    r'self\.leave_scope\(\);\s*HirBlock \{\s*stmts: hir_stmts,\s*span,\s*\}',
    '''if let Some(e) = opt_expr {
            let h_e = self.lower_expr(e);
            hir_stmts.push(HirStmt::Return(Some(h_e), e.span));
        }
        self.leave_scope();
        HirBlock { stmts: hir_stmts, span }''',
    text
)

with open('crates/apl_hir/src/lowering.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print("done")
