with open('crates/apl_hir/src/lowering.rs', 'a', encoding='utf-8') as f:
    f.write('''
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
                    // We can track extern fns in functions for now or a separate map
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
            let _ = self.define_var(param.name.clone(), param.ty.clone(), param.mutability, param.span);
            hir_params.push(HirParam {
                name: param.name.clone(),
                ty: param.ty.clone(),
                span: param.span,
            });
        }
        
        let body = if let Some(b) = &func.body {
            Some(self.lower_block(b))
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
            is_extern: func.body.is_none(),
            span: func.span,
        })
    }

    pub fn lower_block(&mut self, block: &apl_ast::Block) -> HirBlock {
        self.enter_scope();
        let mut stmts = Vec::new();
        for stmt in &block.stmts {
            if let Some(s) = self.lower_stmt(stmt) {
                stmts.push(s);
            }
        }
        self.leave_scope();
        HirBlock {
            stmts,
            span: block.span,
        }
    }

    pub fn lower_stmt(&mut self, stmt: &apl_ast::Stmt) -> Option<HirStmt> {
        match stmt {
            Stmt::Let(decl) => {
                let init = if let Some(e) = &decl.init {
                    Some(self.lower_expr(e))
                } else {
                    None
                };
                let _ = self.define_var(decl.name.clone(), decl.ty.clone(), decl.mutability, decl.span);
                Some(HirStmt::Let {
                    name: decl.name.clone(),
                    mutability: decl.mutability,
                    ty: decl.ty.clone(),
                    init,
                    span: decl.span,
                })
            }
            Stmt::Expr(expr) | Stmt::ExprSemi(expr) => {
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
''')
print("done")
