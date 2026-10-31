import re

with open('crates/apl_hir/src/lowering.rs', 'r', encoding='utf-8') as f:
    text = f.read()

start_idx = text.find('    pub fn lower_expr(')
end_idx = text.rfind('}')

lower_expr_impl = '''    pub fn lower_expr(&mut self, expr: &apl_ast::Expr) -> HirExpr {
        match &expr.kind {
            ExprKind::Ident(name) => {
                if let Some(var) = self.get_var_clone(name) {
                    HirExpr {
                        kind: HirExprKind::LocalRef(var.id),
                        ty: var.ty,
                        span: expr.span,
                    }
                } else {
                    self.errors.push(HirError::UndefinedSymbol(name.clone(), expr.span));
                    HirExpr {
                        kind: HirExprKind::LiteralInt(0),
                        ty: Type::Void,
                        span: expr.span,
                    }
                }
            }
            ExprKind::Literal(lit) => {
                let (kind, ty) = match lit {
                    apl_ast::Literal::Int(v) => (HirExprKind::LiteralInt(*v), Type::I32),
                    apl_ast::Literal::Float(v) => (HirExprKind::LiteralFloat(*v), Type::F64),
                    apl_ast::Literal::Bool(v) => (HirExprKind::LiteralBool(*v), Type::Bool),
                    apl_ast::Literal::String(v) => (HirExprKind::LiteralString(v.clone()), Type::String),
                    apl_ast::Literal::Char(_) => (HirExprKind::LiteralInt(0), Type::I8),
                };
                HirExpr { kind, ty, span: expr.span }
            }
            ExprKind::Binary(op, left, right) => {
                let h_left = self.lower_expr(left);
                let h_right = self.lower_expr(right);
                let ty = match op {
                    apl_ast::BinaryOp::Add | apl_ast::BinaryOp::Sub | apl_ast::BinaryOp::Mul | apl_ast::BinaryOp::Div | apl_ast::BinaryOp::Mod => h_left.ty.clone(),
                    _ => Type::Bool,
                };
                HirExpr {
                    kind: HirExprKind::BinaryOp(op.clone(), Box::new(h_left), Box::new(h_right)),
                    ty,
                    span: expr.span,
                }
            }
            ExprKind::Unary(op, right) => {
                let h_right = self.lower_expr(right);
                let ty = h_right.ty.clone();
                HirExpr {
                    kind: HirExprKind::UnaryOp(op.clone(), Box::new(h_right)),
                    ty,
                    span: expr.span,
                }
            }
            ExprKind::Call(callee, args) => {
                let mut hir_args = Vec::new();
                for arg in args {
                    hir_args.push(self.lower_expr(arg));
                }
                
                if let ExprKind::Ident(name) = &callee.kind {
                    if let Some(&func_id) = self.func_map.get(name) {
                        let ret_ty = if let Some(f) = self.functions.get(name) {
                            f.return_type.clone()
                        } else {
                            Type::Void
                        };
                        return HirExpr {
                            kind: HirExprKind::Call {
                                callee: func_id,
                                args: hir_args,
                            },
                            ty: ret_ty,
                            span: expr.span,
                        };
                    } else {
                        self.errors.push(HirError::UndefinedSymbol(name.clone(), expr.span));
                    }
                }
                
                HirExpr {
                    kind: HirExprKind::LiteralInt(0),
                    ty: Type::Void,
                    span: expr.span,
                }
            }
            ExprKind::Assign(place, rvalue) => {
                let h_place = self.lower_expr(place);
                let h_rvalue = self.lower_expr(rvalue);
                if h_place.ty != h_rvalue.ty {
                    self.errors.push(HirError::TypeMismatch(h_place.ty.clone(), h_rvalue.ty.clone(), expr.span));
                }
                HirExpr {
                    kind: HirExprKind::Assign(Box::new(h_place), Box::new(h_rvalue)),
                    ty: Type::Void,
                    span: expr.span,
                }
            }
            ExprKind::If(cond, then_b, else_b) => {
                let h_cond = self.lower_expr(cond);
                if h_cond.ty != Type::Bool {
                    self.errors.push(HirError::TypeMismatch(Type::Bool, h_cond.ty.clone(), cond.span));
                }
                let h_then = self.lower_expr(then_b);
                let h_else = if let Some(e) = else_b {
                    Some(Box::new(self.lower_expr(e)))
                } else {
                    None
                };
                let ret_ty = h_then.ty.clone(); // simplifed
                HirExpr {
                    kind: HirExprKind::If { cond: Box::new(h_cond), then_branch: Box::new(h_then), else_branch: h_else },
                    ty: ret_ty,
                    span: expr.span,
                }
            }
            ExprKind::While(cond, body) => {
                let h_cond = self.lower_expr(cond);
                if h_cond.ty != Type::Bool {
                    self.errors.push(HirError::TypeMismatch(Type::Bool, h_cond.ty.clone(), cond.span));
                }
                let h_body = self.lower_expr(body);
                HirExpr {
                    kind: HirExprKind::While { cond: Box::new(h_cond), body: Box::new(h_body) },
                    ty: Type::Void,
                    span: expr.span,
                }
            }
            ExprKind::StructInit(name, fields) => {
                let mut hir_fields = Vec::new();
                for (fname, fexpr) in fields {
                    hir_fields.push((fname.clone(), self.lower_expr(fexpr)));
                }
                if let Some(&struct_id) = self.struct_map.get(name) {
                    return HirExpr {
                        kind: HirExprKind::StructInit { struct_id, fields: hir_fields },
                        ty: Type::Named(name.clone(), vec![]),
                        span: expr.span,
                    };
                }
                self.errors.push(HirError::UndefinedSymbol(name.clone(), expr.span));
                HirExpr {
                    kind: HirExprKind::LiteralInt(0),
                    ty: Type::Void,
                    span: expr.span,
                }
            }
            ExprKind::FieldAccess(base, field_name) => {
                let h_base = self.lower_expr(base);
                let mut f_ty = Type::Void;
                if let Type::Named(struct_name, _) = &h_base.ty {
                    if let Some(st) = self.structs.get(struct_name) {
                        if let Some(f) = st.fields.iter().find(|f| f.name == *field_name) {
                            f_ty = f.ty.clone();
                        } else {
                            self.errors.push(HirError::UnknownField(struct_name.clone(), field_name.clone(), expr.span));
                        }
                    }
                }
                HirExpr {
                    kind: HirExprKind::FieldAccess(Box::new(h_base), field_name.clone()),
                    ty: f_ty,
                    span: expr.span,
                }
            }
            ExprKind::MethodCall(receiver, method_name, args) => {
                let h_receiver = self.lower_expr(receiver);
                let mut hir_args = Vec::new();
                for arg in args {
                    hir_args.push(self.lower_expr(arg));
                }
                
                let mut ret_ty = Type::Void;
                let mut m_id = FunctionId(0);
                
                if let Type::Named(struct_name, _) = &h_receiver.ty {
                    let full_name = format!("{}_{}", struct_name, method_name);
                    if let Some(&func_id) = self.func_map.get(&full_name) {
                        m_id = func_id;
                        if let Some(f) = self.functions.get(&full_name) {
                            ret_ty = f.return_type.clone();
                        }
                    } else {
                        self.errors.push(HirError::UndefinedSymbol(full_name.clone(), expr.span));
                    }
                }
                
                HirExpr {
                    kind: HirExprKind::MethodCall {
                        receiver: Box::new(h_receiver),
                        method: m_id,
                        args: hir_args,
                    },
                    ty: ret_ty,
                    span: expr.span,
                }
            }
            ExprKind::Block(stmts, opt_expr) => {
                let h_block = self.lower_block(stmts, expr.span);
                let mut ret_ty = Type::Void;
                let mut h_opt = None;
                if let Some(e) = opt_expr {
                    let h_e = self.lower_expr(e);
                    ret_ty = h_e.ty.clone();
                    h_opt = Some(Box::new(h_e));
                }
                HirExpr {
                    kind: HirExprKind::Block(h_block, h_opt),
                    ty: ret_ty,
                    span: expr.span,
                }
            }
            _ => {
                HirExpr {
                    kind: HirExprKind::LiteralInt(0),
                    ty: Type::Void,
                    span: expr.span,
                }
            }
        }
    }
}
'''
with open('crates/apl_hir/src/lowering.rs', 'w', encoding='utf-8') as f:
    f.write(text[:start_idx] + lower_expr_impl)
print("done")
