
use crate::*;
use apl_ast::{ExprKind, Item, Program, Span, Stmt, Type};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum HirError {
    #[error("Undefined symbol '{0}' at {1:?}")]
    UndefinedSymbol(String, Span),
    #[error("Type mismatch: expected '{0:?}', got '{1:?}' at {2:?}")]
    TypeMismatch(Type, Type, Span),
    #[error("Cannot assign to immutable variable '{0}' at {1:?}")]
    ImmutableAssignment(String, Span),
    #[error("Duplicate symbol definition '{0}' at {1:?}")]
    DuplicateSymbol(String, Span),
    #[error("Missing field '{1}' in struct '{0}' at {2:?}")]
    MissingField(String, String, Span),
    #[error("Unknown field '{1}' in struct '{0}' at {2:?}")]
    UnknownField(String, String, Span),
    #[error("Break or continue outside of loop at {0:?}")]
    BreakOutsideLoop(Span),
}

#[derive(Debug, Clone)]
pub struct VarInfo {
    pub id: LocalId,
    pub name: String,
    pub ty: Type,
    pub mutability: bool,
    pub span: Span,
}

pub struct Scope {
    pub vars: HashMap<String, VarInfo>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }
}

pub struct HirLowering {
    pub errors: Vec<HirError>,
    pub scopes: Vec<Scope>,
    pub functions: HashMap<String, apl_ast::Function>,
    pub structs: HashMap<String, apl_ast::StructDecl>,
    pub enums: HashMap<String, apl_ast::EnumDecl>,
    pub current_package: String,
    pub current_fn_return_type: Option<Type>,
    pub resolved_names: HashMap<Span, String>,

    pub next_local_id: u32,
    pub next_func_id: u32,
    pub next_struct_id: u32,
    pub next_enum_id: u32,

    pub func_map: HashMap<String, FunctionId>,
    pub struct_map: HashMap<String, StructId>,
    pub enum_map: HashMap<String, EnumId>,
}

impl HirLowering {
    pub fn resolve_self_type(ty: &Type, target_struct: &str) -> Type {
        match ty {
            Type::Named(name, _) if name == "Self" => Type::Named(target_struct.to_string(), vec![]),
            Type::Ref(inner) => Type::Ref(Box::new(Self::resolve_self_type(inner, target_struct))),
            Type::RefMut(inner) => Type::RefMut(Box::new(Self::resolve_self_type(inner, target_struct))),
            other => other.clone(),
        }
    }
    fn is_compatible(expected: &Type, actual: &Type) -> bool {
        if expected == actual {
            return true;
        }
        match (expected, actual) {
            (Type::I64, Type::I32) | (Type::I32, Type::I64) => true,
            (Type::U8, Type::I32) | (Type::I32, Type::U8) => true,
            (Type::F64, Type::F32) | (Type::F32, Type::F64) => true,
            (Type::Ref(inner), actual) | (Type::RefMut(inner), actual) => {
                Self::is_compatible(inner, actual)
            }
            (expected, Type::Ref(inner)) | (expected, Type::RefMut(inner)) => {
                Self::is_compatible(expected, inner)
            }
            _ => false,
        }
    }

    pub fn new() -> Self {
        let mut func_map = HashMap::new();
        func_map.insert("print".to_string(), FunctionId(99000));
        func_map.insert("println".to_string(), FunctionId(99001));
        func_map.insert("eprint".to_string(), FunctionId(99002));
        func_map.insert("mmio_write".to_string(), FunctionId(99010));
        func_map.insert("mmio_read".to_string(), FunctionId(99011));
        func_map.insert("bit_set".to_string(), FunctionId(99012));
        func_map.insert("bit_clear".to_string(), FunctionId(99013));
        func_map.insert("mmio_write".to_string(), FunctionId(99010));
        func_map.insert("mmio_read".to_string(), FunctionId(99011));
        func_map.insert("bit_set".to_string(), FunctionId(99012));
        func_map.insert("bit_clear".to_string(), FunctionId(99013));

        Self {
            errors: Vec::new(),
            scopes: Vec::new(),
            functions: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            current_package: String::new(),
            current_fn_return_type: None,
            resolved_names: HashMap::new(),
            next_local_id: 1,
            next_func_id: 0,
            next_struct_id: 0,
            next_enum_id: 0,
            func_map,
            struct_map: HashMap::new(),
            enum_map: HashMap::new(),
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn leave_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn define_var(&mut self, name: String, ty: Type, mutability: bool, span: Span) -> Result<LocalId, ()> {
        let id = LocalId(self.next_local_id);
        self.next_local_id += 1;

        if let Some(scope) = self.scopes.last_mut() {
            if scope.vars.contains_key(&name) {
                self.errors.push(HirError::DuplicateSymbol(name, span));
                return Err(());
            }
            scope.vars.insert(
                name.clone(),
                VarInfo {
                    id,
                    name,
                    ty,
                    mutability,
                    span,
                },
            );
        }
        Ok(id)
    }

    fn lookup_var(&mut self, name: &str) -> Option<&mut VarInfo> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var) = scope.vars.get_mut(name) {
                return Some(var);
            }
        }
        None
    }

    fn get_var_clone(&mut self, name: &str) -> Option<VarInfo> {
        self.lookup_var(name).cloned()
    }

    fn define_func(&mut self, name: String) -> FunctionId {
        if let Some(&id) = self.func_map.get(&name) {
            id
        } else {
            let id = FunctionId(self.next_func_id);
            self.next_func_id += 1;
            self.func_map.insert(name, id);
            id
        }
    }

    pub fn lower_program(&mut self, prog: &Program) -> Result<HirProgram, Vec<HirError>> {
        for item in &prog.items {
            match item {
                Item::Package(name, _) => {
                    self.current_package = name.clone();
                }
                Item::Function(func) => {
                    self.functions.insert(func.name.clone(), func.clone());
                    self.define_func(func.name.clone());
                }
                Item::Struct(st) => {
                    self.structs.insert(st.name.clone(), st.clone());
                    let id = StructId(self.next_struct_id);
                    self.next_struct_id += 1;
                    self.struct_map.insert(st.name.clone(), id);
                    for mut method in st.methods.clone() {
                        method.name = format!("{}_{}", st.name, method.name);
                        for param in &mut method.params {
                            param.ty = Self::resolve_self_type(&param.ty, &st.name);
                        }
                        method.return_type = Self::resolve_self_type(&method.return_type, &st.name);
                        self.functions.insert(method.name.clone(), method.clone());
                        self.define_func(method.name.clone());
                    }
                }
                Item::Enum(en) => {
                    self.enums.insert(en.name.clone(), en.clone());
                    let id = EnumId(self.next_enum_id);
                    self.next_enum_id += 1;
                    self.enum_map.insert(en.name.clone(), id);
                }
                Item::ExternBlock(decls, _) => {
                    for d in decls {
                        self.define_func(d.name.clone());
                    }
                }
                Item::Impl(imp) => {
                    for mut method in imp.methods.clone() {
                        method.name = format!("{}_{}", imp.target_type, method.name);
                        for param in &mut method.params {
                            param.ty = Self::resolve_self_type(&param.ty, &imp.target_type);
                        }
                        method.return_type = Self::resolve_self_type(&method.return_type, &imp.target_type);
                        self.functions.insert(method.name.clone(), method.clone());
                        self.define_func(method.name.clone());
                    }
                }
                _ => {}
            }
        }
        
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
            if let Item::Struct(st) = item {
                for mut method in st.methods.clone() {
                    method.name = format!("{}_{}", st.name, method.name);
                    for param in &mut method.params {
                        param.ty = Self::resolve_self_type(&param.ty, &st.name);
                    }
                    method.return_type = Self::resolve_self_type(&method.return_type, &st.name);
                    if let Some(hf) = self.lower_function(&method) {
                        hir_funcs.push(hf);
                    }
                }
            }
            if let Item::Impl(imp) = item {
                for mut method in imp.methods.clone() {
                    method.name = format!("{}_{}", imp.target_type, method.name);
                    for param in &mut method.params {
                        param.ty = Self::resolve_self_type(&param.ty, &imp.target_type);
                    }
                    method.return_type = Self::resolve_self_type(&method.return_type, &imp.target_type);
                    if let Some(hf) = self.lower_function(&method) {
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
        let func_id = self.func_map.get(&func.name).copied().unwrap();
        
        let mut hir_params = Vec::new();
        for param in &func.params {
            let id = self.define_var(param.name.clone(), param.ty.clone(), false, param.span).unwrap_or(LocalId(0));
            hir_params.push(HirParam {
                id,
                name: param.name.clone(),
                ty: param.ty.clone(),
                span: param.span,
            });
        }
        
        let body = if let ExprKind::Block(stmts, opt_expr) = &func.body.kind {
            Some(self.lower_block(stmts, opt_expr.as_ref(), func.body.span, true))
        } else {
            None
        };
        
        self.leave_scope();
        self.current_fn_return_type = None;
        
        Some(HirFunction {
            id: func_id,
            name: func.name.clone(),
            params: hir_params,
            return_type: func.return_type.clone(),
            body,
            is_extern: false,
            span: func.span,
        })
    }

    pub fn lower_block(&mut self, stmts: &[apl_ast::Stmt], opt_expr: Option<&Box<apl_ast::Expr>>, span: Span, is_func_body: bool) -> HirBlock {
        self.enter_scope();
        let mut hir_stmts = Vec::new();
        for stmt in stmts {
            if let Some(s) = self.lower_stmt(stmt) {
                hir_stmts.push(s);
            }
        }
        if let Some(e) = opt_expr {
            let h_e = self.lower_expr(e);
            if is_func_body && self.current_fn_return_type.as_ref().map(|t| *t != Type::Void).unwrap_or(false) {
                hir_stmts.push(HirStmt::Return(Some(h_e), e.span));
            } else {
                hir_stmts.push(HirStmt::Expr(h_e));
            }
        }
        self.leave_scope();
        HirBlock { stmts: hir_stmts, span }
    }

    pub fn lower_stmt(&mut self, stmt: &apl_ast::Stmt) -> Option<HirStmt> {
        match stmt {
                        Stmt::Let { name, ty, mutability, init, span } => {
                let hir_init = self.lower_expr(init);
                let defined_ty = ty.clone().unwrap_or(hir_init.ty.clone());
                let id = self.define_var(name.clone(), defined_ty.clone(), *mutability, *span).unwrap_or(LocalId(0));
                Some(HirStmt::Let {
                    id,
                    name: name.clone(),
                    mutability: *mutability,
                    ty: defined_ty,
                    init: Some(hir_init),
                    span: *span,
                })
            }
            Stmt::Expr(expr) => {
                if let ExprKind::Return(opt_val) = &expr.kind {
                    let ret_expr = opt_val.as_ref().map(|e| self.lower_expr(e));
                    Some(HirStmt::Return(ret_expr, expr.span))
                } else {
                    Some(HirStmt::Expr(self.lower_expr(expr)))
                }
            }
        }
    }

    pub fn lower_expr(&mut self, expr: &apl_ast::Expr) -> HirExpr {
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
                    apl_ast::BinaryOp::Add
                    | apl_ast::BinaryOp::Sub
                    | apl_ast::BinaryOp::Mul
                    | apl_ast::BinaryOp::Div
                    | apl_ast::BinaryOp::Mod
                    | apl_ast::BinaryOp::BitAnd
                    | apl_ast::BinaryOp::BitOr
                    | apl_ast::BinaryOp::BitXor
                    | apl_ast::BinaryOp::Shl
                    | apl_ast::BinaryOp::Shr => h_left.ty.clone(),
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
                let ty = match op {
                    apl_ast::UnaryOp::Ref => Type::Ref(Box::new(h_right.ty.clone())),
                    apl_ast::UnaryOp::RefMut => Type::RefMut(Box::new(h_right.ty.clone())),
                    apl_ast::UnaryOp::Deref => match &h_right.ty {
                        Type::Ref(inner) | Type::RefMut(inner) => (**inner).clone(),
                        _ => h_right.ty.clone(),
                    },
                    apl_ast::UnaryOp::Not => Type::Bool,
                    _ => h_right.ty.clone(),
                };
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
                    if name == "print" || name == "println" || name == "eprint" {
                        let callee_id = match name.as_str() {
                            "print" => FunctionId(99000),
                            "println" => FunctionId(99001),
                            _ => FunctionId(99002),
                        };
                        return HirExpr {
                            kind: HirExprKind::Call {
                                callee: callee_id,
                                args: hir_args,
                            },
                            ty: Type::Void,
                            span: expr.span,
                        };
                    }
                    if name == "mmio_write" || name == "bit_set" || name == "bit_clear" {
                        let callee_id = match name.as_str() {
                            "mmio_write" => FunctionId(99010),
                            "bit_set" => FunctionId(99012),
                            _ => FunctionId(99013),
                        };
                        return HirExpr {
                            kind: HirExprKind::Call {
                                callee: callee_id,
                                args: hir_args,
                            },
                            ty: Type::Void,
                            span: expr.span,
                        };
                    }
                    if name == "mmio_read" {
                        return HirExpr {
                            kind: HirExprKind::Call {
                                callee: FunctionId(99011),
                                args: hir_args,
                            },
                            ty: Type::I32,
                            span: expr.span,
                        };
                    }
                    if name == "mmio_write" || name == "bit_set" || name == "bit_clear" {
                        let callee_id = match name.as_str() {
                            "mmio_write" => FunctionId(99010),
                            "bit_set" => FunctionId(99012),
                            _ => FunctionId(99013),
                        };
                        return HirExpr {
                            kind: HirExprKind::Call {
                                callee: callee_id,
                                args: hir_args,
                            },
                            ty: Type::Void,
                            span: expr.span,
                        };
                    }
                    if name == "mmio_read" {
                        return HirExpr {
                            kind: HirExprKind::Call {
                                callee: FunctionId(99011),
                                args: hir_args,
                            },
                            ty: Type::I32,
                            span: expr.span,
                        };
                    }
                    let full_pkg = format!("{}::{}", self.current_package, name);
                    let target_name = if self.func_map.contains_key(name) {
                        Some(name.as_str())
                    } else if !self.current_package.is_empty() && self.func_map.contains_key(&full_pkg) {
                        Some(full_pkg.as_str())
                    } else {
                        None
                    };

                    if let Some(target) = target_name {
                        let func_id = *self.func_map.get(target).unwrap();
                        let ret_ty = if let Some(f) = self.functions.get(target) {
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
                if !Self::is_compatible(&h_place.ty, &h_rvalue.ty) {
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

                let ret_ty = match &h_else {
                    Some(e) if h_then.ty == e.ty => h_then.ty.clone(),
                    Some(e) if h_then.ty == Type::Void || e.ty == Type::Void => {
                        if h_then.ty == Type::Void { e.ty.clone() } else { h_then.ty.clone() }
                    }
                    Some(e) => {
                        self.errors.push(HirError::TypeMismatch(h_then.ty.clone(), e.ty.clone(), expr.span));
                        h_then.ty.clone()
                    }
                    None => h_then.ty.clone(),
                };

                HirExpr {
                    kind: HirExprKind::If { cond: Box::new(h_cond), then_branch: Box::new(h_then), else_branch: h_else },
                    ty: ret_ty,
                    span: expr.span,
                }
            }

            ExprKind::Loop(body) => {
                let h_body = self.lower_expr(body);
                HirExpr {
                    kind: HirExprKind::Loop(Box::new(h_body)),
                    ty: Type::Void,
                    span: expr.span,
                }
            }
            ExprKind::Break => {
                HirExpr {
                    kind: HirExprKind::Break,
                    ty: Type::Void,
                    span: expr.span,
                }
            }
            ExprKind::Continue => {
                HirExpr {
                    kind: HirExprKind::Continue,
                    ty: Type::Void,
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
                let mut struct_name_opt = None;
                if let Type::Named(s, _) = &h_base.ty {
                    struct_name_opt = Some(s.clone());
                } else if let Type::Ref(inner) | Type::RefMut(inner) = &h_base.ty {
                    if let Type::Named(s, _) = &**inner {
                        struct_name_opt = Some(s.clone());
                    }
                }

                if let Some(struct_name) = struct_name_opt {
                    if let Some(st) = self.structs.get(&struct_name) {
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
                
                let mut struct_name_opt = None;
                if let Type::Named(s, _) = &h_receiver.ty {
                    struct_name_opt = Some(s.clone());
                } else if let Type::Ref(inner) | Type::RefMut(inner) = &h_receiver.ty {
                    if let Type::Named(s, _) = &**inner {
                        struct_name_opt = Some(s.clone());
                    }
                }

                if let Some(struct_name) = struct_name_opt {
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
                let h_block = self.lower_block(stmts, opt_expr.as_ref(), expr.span, false);
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
            ExprKind::Return(opt_val) => {
                let ret_expr = opt_val.as_ref().map(|e| Box::new(self.lower_expr(e)));
                let ret_ty = ret_expr.as_ref().map(|e| e.ty.clone()).unwrap_or(Type::Void);
                let h_block = HirBlock { stmts: Vec::new(), span: expr.span };
                HirExpr {
                    kind: HirExprKind::Block(h_block, ret_expr),
                    ty: ret_ty,
                    span: expr.span,
                }
            }
            ExprKind::Index(base, index) => {
                let h_base = self.lower_expr(base);
                let h_index = self.lower_expr(index);
                let elem_ty = match &h_base.ty {
                    Type::Array(elem, _) => (**elem).clone(),
                    Type::Ref(inner) | Type::RefMut(inner) => match &**inner {
                        Type::Array(elem, _) => (**elem).clone(),
                        _ => Type::I32,
                    },
                    _ => Type::I32,
                };
                HirExpr {
                    kind: HirExprKind::Index(Box::new(h_base), Box::new(h_index)),
                    ty: elem_ty,
                    span: expr.span,
                }
            }
            ExprKind::ArrayInit(items) => {
                let h_items: Vec<HirExpr> = items.iter().map(|item| self.lower_expr(item)).collect();
                let elem_ty = if let Some(first) = h_items.first() {
                    first.ty.clone()
                } else {
                    Type::I32
                };
                let len = h_items.len();
                HirExpr {
                    kind: HirExprKind::ArrayInit(h_items),
                    ty: Type::Array(Box::new(elem_ty), len),
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
