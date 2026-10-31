import re

with open('crates/apl_mir/src/lowering.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# 1. Add scopes to MirBuilder struct
text = text.replace('loop_scopes: Vec<(BasicBlockId, BasicBlockId)>,', 'loop_scopes: Vec<(BasicBlockId, BasicBlockId)>,\n    scopes: Vec<Vec<LocalId>>,')

# 2. Add scopes to MirBuilder::new
text = text.replace('loop_scopes: Vec::new(),', 'loop_scopes: Vec::new(),\n            scopes: Vec::new(),')

# 3. Modify add_local
func_add = '''    fn add_local(&mut self, id: LocalId, ty: Type, span: Span, is_mut: bool, is_param: bool) {
        self.locals.push(LocalDecl { id, ty, span, is_mut, is_param });
        if !self.scopes.is_empty() {
            let last_idx = self.scopes.len() - 1;
            self.scopes[last_idx].push(id);
        }
    }'''
text = re.sub(r'    fn add_local\(&mut self, id: LocalId, ty: Type, span: Span, is_mut: bool, is_param: bool\) \{\n        self\.locals\.push\(LocalDecl \{ id, ty, span, is_mut, is_param \}\);\n    \}', func_add, text)

# 4. Modify lower_block to push and pop scopes and inject Drops
func_block = '''    fn lower_block(&mut self, block: &HirBlock) {
        self.scopes.push(Vec::new());
        for stmt in &block.stmts {
            self.lower_stmt(stmt);
        }
        let dropped_locals = self.scopes.pop().unwrap();
        for local_id in dropped_locals.into_iter().rev() {
            // Find the local decl to check if it's Copy
            let is_copy = self.locals.iter().find(|l| l.id == local_id).map(|l| l.ty.is_copy()).unwrap_or(true);
            if !is_copy {
                let span = block.span; // fallback span
                self.blocks[self.current_block.0 as usize].statements.push(Statement::Drop(Place::Local(local_id), span));
            }
            // Always inject StorageDead
            let span = block.span;
            self.blocks[self.current_block.0 as usize].statements.push(Statement::StorageDead(local_id, span));
        }
    }'''
text = re.sub(r'    fn lower_block\(&mut self, block: &HirBlock\) \{\n        for stmt in &block\.stmts \{\n            self\.lower_stmt\(stmt\);\n        \}\n    \}', func_block, text)

with open('crates/apl_mir/src/lowering.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("done")
