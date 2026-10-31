with open('crates/apl_hir/src/lowering.rs', 'r', encoding='utf-8') as f:
    lines = f.read().splitlines()

# find the closing brace of impl HirLowering
# The last brace in the file
content = '\n'.join(lines[:-1]) # remove the last '}'

content += '''
    pub fn lower_program(&mut self, prog: &Program) -> Result<HirProgram, Vec<HirError>> {
        for item in &prog.items {
            match item {
                Item::Package(name, _) => {
                    self.current_package = name.clone();
                }
                Item::Function(func) => {
                    self.functions.insert(func.name.clone(), func.clone());
                }
                Item::Struct(st) => {
                    self.structs.insert(st.name.clone(), st.clone());
                }
                Item::Enum(en) => {
                    self.enums.insert(en.name.clone(), en.clone());
                }
                Item::ExternBlock(decls, _) => {
                    // extern
                }
                Item::Impl(imp) => {
                    for mut method in imp.methods.clone() {
                        method.name = format!("{}_{}", imp.target_type, method.name);
                        self.functions.insert(method.name.clone(), method);
                    }
                }
                _ => {}
            }
        }
        
        // Actually lower functions
        let mut hir_funcs = Vec::new();
        self.current_package.clear();
        for item in &prog.items {
            if let Item::Package(name, _) = item {
                self.current_package = name.clone();
            }
            if let Item::Function(func) = item {
                if let Some(hf) = self.lower_function(func) {
                    hir_funcs.push(hf);
                }
            }
            if let Item::Impl(imp) = item {
                for method in &imp.methods {
                    if let Some(hf) = self.lower_function(method) {
                        hir_funcs.push(hf);
                    }
                }
            }
        }
        
        if self.errors.is_empty() {
            Ok(HirProgram {
                functions: hir_funcs,
                structs: vec![],
                enums: vec![],
            })
        } else {
            Err(self.errors.clone())
        }
    }

    pub fn lower_function(&mut self, func: &apl_ast::Function) -> Option<HirFunction> {
        self.enter_scope();
        self.current_fn_return_type = Some(func.return_type.clone());
        
        let mut hir_params = Vec::new();
        for param in &func.params {
            let _ = self.define_var(param.name.clone(), param.ty.clone(), false, param.span);
            hir_params.push(HirParam {
                name: param.name.clone(),
                ty: param.ty.clone(),
                span: param.span,
            });
        }
        
        let body = if let ExprKind::Block(stmts) = &func.body.kind {
            Some(self.lower_block(stmts, func.body.span))
        } else {
            None
        };
        
        self.leave_scope();
        self.current_fn_return_type = None;
        
        Some(HirFunction {
            name: func.name.clone(),
            params: hir_params,
            return_type: func.return_type.clone(),
            body,
            is_extern: false,
            span: func.span,
        })
    }

    pub fn lower_block(&mut self, stmts: &[apl_ast::Stmt], span: Span) -> HirBlock {
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
    }

    pub fn lower_stmt(&mut self, stmt: &apl_ast::Stmt) -> Option<HirStmt> {
        match stmt {
            Stmt::Let { name, ty, mutability, init, span } => {
                let hir_init = if let Some(e) = init {
                    Some(self.lower_expr(e))
                } else {
                    None
                };
                let _ = self.define_var(name.clone(), ty.clone(), *mutability, *span);
                Some(HirStmt::Let {
                    name: name.clone(),
                    mutability: *mutability,
                    ty: ty.clone(),
                    init: hir_init,
                    span: *span,
                })
            }
            Stmt::Expr(expr) => {
                Some(HirStmt::Expr(self.lower_expr(expr)))
            }
            Stmt::Return(opt_expr, span) => {
                let expr = opt_expr.as_ref().map(|e| self.lower_expr(e));
                Some(HirStmt::Return(expr, *span))
            }
            _ => None
        }
    }

    pub fn lower_expr(&mut self, expr: &apl_ast::Expr) -> HirExpr {
        // Dummy implementation for now, just to compile
        HirExpr {
            kind: HirExprKind::LiteralInt(0),
            ty: Type::I32,
            span: expr.span,
        }
    }
}
'''
with open('crates/apl_hir/src/lowering.rs', 'w', encoding='utf-8') as f:
    f.write(content)
print("done")
