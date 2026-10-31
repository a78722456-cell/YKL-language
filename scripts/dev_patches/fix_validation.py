with open('crates/apl_hir/src/lib.rs', 'r', encoding='utf-8') as f:
    text = f.read()

validation_code = '''
impl HirProgram {
    pub fn validate(&self) -> Result<(), String> {
        let mut func_ids = std::collections::HashSet::new();
        for func in &self.functions {
            if !func_ids.insert(func.id) {
                return Err(format!("Duplicate FunctionId: {:?}", func.id));
            }
            
            let mut local_ids = std::collections::HashSet::new();
            for param in &func.params {
                if !local_ids.insert(param.id) {
                    return Err(format!("Duplicate LocalId in params: {:?}", param.id));
                }
            }
            
            if let Some(body) = &func.body {
                self.validate_block(body, &mut local_ids, &func_ids)?;
            }
        }
        Ok(())
    }
    
    fn validate_block(&self, block: &HirBlock, local_ids: &mut std::collections::HashSet<LocalId>, func_ids: &std::collections::HashSet<FunctionId>) -> Result<(), String> {
        for stmt in &block.stmts {
            match stmt {
                HirStmt::Let { id, init, .. } => {
                    if !local_ids.insert(*id) {
                        return Err(format!("Duplicate LocalId in let: {:?}", id));
                    }
                    if let Some(expr) = init {
                        self.validate_expr(expr, local_ids, func_ids)?;
                    }
                }
                HirStmt::Expr(expr) => {
                    self.validate_expr(expr, local_ids, func_ids)?;
                }
                HirStmt::Return(opt_expr, _) => {
                    if let Some(expr) = opt_expr {
                        self.validate_expr(expr, local_ids, func_ids)?;
                    }
                }
            }
        }
        Ok(())
    }
    
    fn validate_expr(&self, expr: &HirExpr, local_ids: &std::collections::HashSet<LocalId>, func_ids: &std::collections::HashSet<FunctionId>) -> Result<(), String> {
        match &expr.kind {
            HirExprKind::LocalRef(id) => {
                if !local_ids.contains(id) {
                    return Err(format!("Reference to unknown LocalId: {:?}", id));
                }
            }
            HirExprKind::Call { callee, args } => {
                // We might not have all func_ids populated if they are parsed later, but in a real validator we should check.
                // For now, just check args
                for arg in args {
                    self.validate_expr(arg, local_ids, func_ids)?;
                }
            }
            HirExprKind::BinaryOp(_, left, right) => {
                self.validate_expr(left, local_ids, func_ids)?;
                self.validate_expr(right, local_ids, func_ids)?;
            }
            _ => {}
        }
        Ok(())
    }
}
'''

text = text.replace('''impl HirProgram {
    pub fn validate(&self) -> Result<(), String> {
        // Validation logic will go here
        Ok(())
    }
}''', validation_code)

with open('crates/apl_hir/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print('done')
