use apl_ast::*;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum SemaError {
    #[error("Undefined symbol '{0}' at {1:?}")]
    UndefinedSymbol(String, Span),
    #[error("Type mismatch: expected '{0:?}', got '{1:?}' at {2:?}")]
    TypeMismatch(Type, Type, Span),
    #[error("Use of moved value '{0}' at {1:?}")]
    UseAfterMove(String, Span),
    #[error("Cannot borrow '{0}' mutably because it is already borrowed at {1:?}")]
    BorrowConflict(String, Span),
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

#[derive(Debug, Clone, PartialEq)]
pub enum OwnershipState {
    Owned,
    Moved(Span),
    BorrowedImm(usize), // count of active immutable borrows
    BorrowedMut(Span),  // location of mutable borrow
}

#[derive(Debug, Clone)]
pub struct VarInfo {
    pub name: String,
    pub ty: Type,
    pub mutability: bool,
    pub ownership: OwnershipState,
    pub escapes: bool,
    pub is_used: bool,
    pub span: Span, // to track where it was defined for the warning
}

#[derive(Debug, Clone, PartialEq)]
pub enum SemaWarning {
    UnusedVariable(String, Span),
    UnusedFunction(String, Span),
    NamingConvention(String, String, Span), // name, expected format (e.g. snake_case)
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

pub struct SemanticAnalyzer {
    pub scopes: Vec<Scope>,
    pub functions: HashMap<String, Function>,
    pub structs: HashMap<String, StructDecl>,
    pub enums: HashMap<String, EnumDecl>,
    pub extern_fns: HashMap<String, ExternDecl>,
    pub errors: Vec<SemaError>,
    pub warnings: Vec<SemaWarning>,
    pub current_fn_return_type: Option<Type>,
    pub last_return_type: Option<Type>,
    pub type_map: HashMap<Span, Type>,
    pub resolved_names: HashMap<Span, String>,
    pub current_package: String,
    is_borrow_context: bool,
    loop_depth: usize,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let mut sa = Self {
            scopes: vec![Scope::new()],
            functions: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            extern_fns: HashMap::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            current_fn_return_type: None,
            last_return_type: None,
            type_map: HashMap::new(),
            resolved_names: HashMap::new(),
            current_package: String::new(),
            is_borrow_context: false,
            loop_depth: 0,
        };
        sa.register_builtins();
        sa
    }

    fn register_builtins(&mut self) {
        let println_func = Function {
            name: "println".to_string(),
            generics: vec!["T".to_string()],
            params: vec![Param {
                name: "msg".to_string(),
                ty: Type::Named("T".to_string(), vec![]),
                span: Span::dummy(),
            }],
            return_type: Type::Void,
            is_async: false,
            is_pub: true,
            body: Expr {
                kind: ExprKind::Block(vec![], None),
                span: Span::dummy(),
            },
            span: Span::dummy(),
        };
        self.functions.insert("println".to_string(), println_func);

        let alloc_array_func = Function {
            name: "alloc_array".to_string(),
            generics: vec!["T".to_string()],
            params: vec![Param {
                name: "size".to_string(),
                ty: Type::I32,
                span: Span::dummy(),
            }, Param {
                name: "dummy".to_string(),
                ty: Type::Ref(Box::new(Type::Named("T".to_string(), vec![]))),
                span: Span::dummy(),
            }],
            return_type: Type::Slice(Box::new(Type::Named("T".to_string(), vec![]))),
            is_async: false,
            is_pub: true,
            body: Expr {
                kind: ExprKind::Block(vec![], None),
                span: Span::dummy(),
            },
            span: Span::dummy(),
        };
        self.functions
            .insert("alloc_array".to_string(), alloc_array_func);
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn leave_scope(&mut self) {
        if let Some(scope) = self.scopes.pop() {
            for (name, var) in scope.vars {
                if !var.is_used && !name.starts_with('_') {
                    self.warnings.push(SemaWarning::UnusedVariable(name.clone(), var.span));
                }
            }
        }
    }

    fn define_var(
        &mut self,
        name: String,
        ty: Type,
        mutability: bool,
        span: Span,
    ) -> Result<(), SemaError> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.vars.contains_key(&name) {
                let err = SemaError::DuplicateSymbol(name.clone(), span);
                self.errors.push(err.clone());
                return Err(err);
            }
            scope.vars.insert(
                name.clone(),
                VarInfo {
                    name,
                    ty,
                    mutability,
                    ownership: OwnershipState::Owned,
                    escapes: false,
                    is_used: false,
                    span,
                },
            );
        }
        Ok(())
    }

    fn lookup_var(&mut self, name: &str) -> Option<&mut VarInfo> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var) = scope.vars.get_mut(name) {
                var.is_used = true;
                return Some(var);
            }
        }
        None
    }

    fn get_var_clone(&mut self, name: &str) -> Option<VarInfo> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var) = scope.vars.get_mut(name) {
                var.is_used = true;
                return Some(var.clone());
            }
        }
        None
    }

    
    fn monomorphize_type(&mut self, ty: &Type, mapping: &HashMap<String, Type>) -> Type {
        match ty {
            Type::Named(name, args) => {
                if args.is_empty() {
                    if let Some(subst) = mapping.get(name) {
                        return subst.clone();
                    }
                }
                let new_args: Vec<Type> = args
                    .iter()
                    .map(|a| self.monomorphize_type(a, mapping))
                    .collect();
                
                if !new_args.is_empty() {
                    let mut type_name = name.clone();
                    for a in &new_args {
                        let arg_str = format!("{}", a).replace(" ", "").replace("<", "_").replace(">", "_").replace(",", "_");
                        type_name.push_str("_");
                        type_name.push_str(&arg_str);
                    }
                    
                    if !self.structs.contains_key(&type_name) && self.structs.contains_key(name) {
                        let orig = self.structs.get(name).unwrap().clone();
                        let mut new_struct = orig.clone();
                        new_struct.name = type_name.clone();
                        new_struct.generics = Vec::new();
                        
                        let mut local_map = HashMap::new();
                        for (i, g) in orig.generics.iter().enumerate() {
                            if let Some(a) = new_args.get(i) {
                                local_map.insert(g.clone(), a.clone());
                            }
                        }
                        for field in &mut new_struct.fields {
                            field.ty = self.monomorphize_type(&field.ty, &local_map);
                        }
                        self.structs.insert(type_name.clone(), new_struct);
                    } else if !self.enums.contains_key(&type_name) && self.enums.contains_key(name) {
                        let orig = self.enums.get(name).unwrap().clone();
                        let mut new_enum = orig.clone();
                        new_enum.name = type_name.clone();
                        new_enum.generics = Vec::new();
                        
                        let mut local_map = HashMap::new();
                        for (i, g) in orig.generics.iter().enumerate() {
                            if let Some(a) = new_args.get(i) {
                                local_map.insert(g.clone(), a.clone());
                            }
                        }
                        for variant in &mut new_enum.variants {
                            for ty in &mut variant.types {
                                *ty = self.monomorphize_type(ty, &local_map);
                            }
                        }
                        self.enums.insert(type_name.clone(), new_enum);
                    }
                    
                    return Type::Named(name.clone(), new_args.clone());
                }
                Type::Named(name.clone(), new_args)
            }
            Type::Ref(inner) => Type::Ref(Box::new(self.monomorphize_type(inner, mapping))),
            Type::RefMut(inner) => Type::RefMut(Box::new(self.monomorphize_type(inner, mapping))),
            Type::Nullable(inner) => Type::Nullable(Box::new(self.monomorphize_type(inner, mapping))),
            Type::Array(inner, size) => Type::Array(Box::new(self.monomorphize_type(inner, mapping)), *size),
            Type::Slice(inner) => Type::Slice(Box::new(self.monomorphize_type(inner, mapping))),
            Type::Channel(inner) => Type::Channel(Box::new(self.monomorphize_type(inner, mapping))),
            _ => ty.clone(),
        }
    }
    


    pub fn analyze_function(&mut self, func: &Function) {
        self.scopes.push(Scope { vars: HashMap::new() });
        for param in &func.params {
            self.define_var(param.name.clone(), param.ty.clone(), false, param.span).unwrap();
        }
        self.current_fn_return_type = Some(func.return_type.clone());
        let ty = self.analyze_expr(&func.body);
        
        if func.return_type != Type::Void && !self.is_compatible(&func.return_type, &ty) {
            self.errors.push(SemaError::TypeMismatch(func.return_type.clone(), ty, func.body.span));
        }

        let scope = self.scopes.pop().unwrap();
        for (_, var_info) in scope.vars {
            if !var_info.is_used && var_info.name != "_" {
                self.warnings.push(SemaWarning::UnusedVariable(var_info.name, var_info.span));
            }
        }
        self.current_fn_return_type = None;
    }
    pub fn analyze_program(&mut self, prog: &Program) -> Result<(), Vec<SemaError>> {
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
                    for decl in decls {
                        self.extern_fns.insert(decl.name.clone(), decl.clone());
                    }
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
        self.current_package.clear();
        for item in &prog.items {
            if let Item::Package(name, _) = item {
                self.current_package = name.clone();
            }
            if let Item::Function(func) = item {
                self.analyze_function(func);
            }
            if let Item::Impl(imp) = item {
                for method in &imp.methods {
                    self.analyze_function(method);
                }
            }
        }
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }
    pub fn analyze_expr(&mut self, expr: &Expr) -> Type {
        let ty = self.analyze_expr_inner(expr);
        self.type_map.insert(expr.span, ty.clone());
        ty
    }

    fn infer_generics(expected: &Type, actual: &Type, mapping: &mut HashMap<String, Type>, generics: &[String]) {
        match (expected, actual) {
            (Type::Named(name, args1), Type::Named(_, args2)) => {
                if args1.is_empty() && generics.contains(name) {
                    mapping.entry(name.clone()).or_insert(actual.clone());
                } else if args1.len() == args2.len() {
                    for (a1, a2) in args1.iter().zip(args2.iter()) {
                        Self::infer_generics(a1, a2, mapping, generics);
                    }
                }
            }
            (Type::Named(name, args), _) => {
                if args.is_empty() && generics.contains(name) {
                    mapping.entry(name.clone()).or_insert(actual.clone());
                }
            }
            (Type::Ref(i1), Type::Ref(i2)) | (Type::RefMut(i1), Type::RefMut(i2)) | (Type::Nullable(i1), Type::Nullable(i2)) | (Type::Slice(i1), Type::Slice(i2)) | (Type::Channel(i1), Type::Channel(i2)) => {
                Self::infer_generics(i1, i2, mapping, generics);
            }
            (Type::Array(i1, s1), Type::Array(i2, s2)) if s1 == s2 => {
                Self::infer_generics(i1, i2, mapping, generics);
            }
            _ => {}
        }
    }

    fn is_compatible(&self, expected: &Type, actual: &Type) -> bool {
        if let (Type::Ptr, Type::I32) | (Type::Ptr, Type::I64) = (expected, actual) { return true; }
        if let (Type::I32, Type::Ptr) | (Type::I64, Type::Ptr) = (expected, actual) { return true; }
        if let (Type::Ptr, Type::String) | (Type::String, Type::Ptr) = (expected, actual) { return true; }
        if let (Type::Slice(_), Type::Ptr) | (Type::Ptr, Type::Slice(_)) = (expected, actual) { return true; }
        if let (Type::Array(_, _), Type::Ptr) | (Type::Ptr, Type::Array(_, _)) = (expected, actual) { return true; }
        
        if expected == actual {
            return true;
        }
        if let (Type::Named(n1, a1), Type::Named(n2, a2)) = (expected, actual) {
            if n1 == n2 && (a1.is_empty() || a2.is_empty()) { return true; }
        }
        if let Type::Named(name, args) = expected {
            if args.is_empty() && (name == "T" || name == "U" || name == "E") { // Hack for unresolved generics
                return true; 
            }
        }
        if let (Type::Ref(i1), Type::RefMut(i2)) = (expected, actual) {
            return self.is_compatible(i1, i2);
        }
        false
    }
fn analyze_expr_inner(&mut self, expr: &Expr) -> Type {
        match &expr.kind {
            ExprKind::Literal(lit) => match lit {
                Literal::Int(_) => Type::I32,
                Literal::Float(_) => Type::F64,
                Literal::Bool(_) => Type::Bool,
                Literal::String(_) => Type::String,
                Literal::Char(_) => Type::Char,
            },
            ExprKind::Ident(name) => {
                if let Some(var) = self.get_var_clone(name) {
                    let var_ty = var.ty.clone();
                    let is_copy = matches!(
                        var.ty,
                        Type::I8
                            | Type::I16
                            | Type::I32
                            | Type::I64
                            | Type::ISize
                            | Type::U8
                            | Type::U16
                            | Type::U32
                            | Type::U64
                            | Type::USize
                            | Type::F32
                            | Type::F64
                            | Type::Bool
                            | Type::Char
                            | Type::Ref(_)
                            | Type::Ptr
                            | Type::String
                    );
                    if let OwnershipState::Moved(_moved_span) = var.ownership {
                        self.errors.push(SemaError::UseAfterMove(name.clone(), expr.span));
                    }
                    if !is_copy && !self.is_borrow_context {
                        if let Some(v_mut) = self.lookup_var(name) {
                            v_mut.ownership = OwnershipState::Moved(expr.span);
                        }
                    }
                    var_ty
                } else if let Some(func) = self.functions.get(name) {
                    func.return_type.clone()
                } else {
                    if let Some((en_name, var_name)) = name.split_once("::") {
                        if let Some(en) = self.enums.get(en_name).cloned() {
                            if en.variants.iter().any(|v| v.name == var_name) {
                                return Type::Named(en_name.to_string(), vec![]);
                            }
                        }
                    }
                    self.errors
                        .push(SemaError::UndefinedSymbol(name.clone(), expr.span));
                    Type::Void
                }
            }
            ExprKind::Assign(left, right) => {
                let r_ty = self.analyze_expr(right);

                if let ExprKind::Ident(name) = &left.kind {
                    if let Some(var) = self.get_var_clone(name) {
                        if !var.mutability {
                            self.errors
                                .push(SemaError::ImmutableAssignment(name.clone(), left.span));
                        }
                        if !self.is_compatible(&var.ty, &r_ty) {
                            self.errors.push(SemaError::TypeMismatch(
                                var.ty.clone(),
                                r_ty.clone(),
                                right.span,
                            ));
                        }
                    } else {
                        self.errors
                            .push(SemaError::UndefinedSymbol(name.clone(), left.span));
                    }
                } else {
                    let l_ty = self.analyze_expr(left);
                    if !self.is_compatible(&l_ty, &r_ty) {
                        self.errors
                            .push(SemaError::TypeMismatch(l_ty, r_ty.clone(), right.span));
                    }
                }
                self.type_map.insert(expr.span, Type::Void);
                return Type::Void;
            }
            ExprKind::Binary(op, left, right) => {
                let lt = self.analyze_expr(left);
                let _rt = self.analyze_expr(right);
                match op {
                    BinaryOp::Eq
                    | BinaryOp::Neq
                    | BinaryOp::Lt
                    | BinaryOp::Lte
                    | BinaryOp::Gt
                    | BinaryOp::Gte
                    | BinaryOp::And
                    | BinaryOp::Or => Type::Bool,
                    _ => lt,
                }
            }
            ExprKind::Unary(op, operand) => match op {
                UnaryOp::Ref => {
                    if let ExprKind::Ident(name) = &operand.kind {
                        if let Some(var) = self.get_var_clone(name) {
                            if let OwnershipState::BorrowedMut(span) = var.ownership {
                                self.errors
                                    .push(SemaError::BorrowConflict(name.clone(), span));
                            } else if let Some(v_mut) = self.lookup_var(name) {
                                match v_mut.ownership {
                                    OwnershipState::BorrowedImm(cnt) => {
                                        v_mut.ownership = OwnershipState::BorrowedImm(cnt + 1)
                                    }
                                    OwnershipState::Owned => {
                                        v_mut.ownership = OwnershipState::BorrowedImm(1)
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    let prev = self.is_borrow_context;
                    self.is_borrow_context = true;
                    let ot = self.analyze_expr(operand);
                    self.is_borrow_context = prev;
                    Type::Ref(Box::new(ot))
                }
                UnaryOp::RefMut => {
                    if let ExprKind::Ident(name) = &operand.kind {
                        if let Some(var) = self.get_var_clone(name) {
                            if !var.mutability {
                                self.errors.push(SemaError::ImmutableAssignment(
                                    name.clone(),
                                    operand.span,
                                ));
                            }
                            if var.ownership != OwnershipState::Owned {
                                self.errors
                                    .push(SemaError::BorrowConflict(name.clone(), operand.span));
                            } else if let Some(v_mut) = self.lookup_var(name) {
                                v_mut.ownership = OwnershipState::BorrowedMut(operand.span);
                            }
                        }
                    }
                    let prev = self.is_borrow_context;
                    self.is_borrow_context = true;
                    let ot = self.analyze_expr(operand);
                    self.is_borrow_context = prev;
                    Type::RefMut(Box::new(ot))
                }
                UnaryOp::Deref => {
                    let ot = self.analyze_expr(operand);
                    match ot {
                        Type::Ref(inner) | Type::RefMut(inner) => *inner,
                        _ => ot,
                    }
                }
                _ => self.analyze_expr(operand),
            },
            ExprKind::Call(callee, args) => {
                let mut arg_types = Vec::new();
                for arg in args {
                    arg_types.push(self.analyze_expr(arg));
                }
                if let ExprKind::Ident(name) = &callee.kind {
                    let mut resolved_name = name.clone();
                    let mut func_opt = self.functions.get(&resolved_name).cloned();
                    if func_opt.is_none() && !self.current_package.is_empty() {
                        resolved_name = format!("{}::{}", self.current_package, name);
                        func_opt = self.functions.get(&resolved_name).cloned();
                    }

                    if let Some(func) = func_opt {
                        self.resolved_names.insert(callee.span, resolved_name.clone());
                        let mut type_mapping = HashMap::new();

                        // Infer generics from arguments
                        for (i, param) in func.params.iter().enumerate() {
                            if let Some(actual_ty) = arg_types.get(i) {
                                Self::infer_generics(
                                    &param.ty,
                                    actual_ty,
                                    &mut type_mapping,
                                    &func.generics,
                                );
                            }
                        }

                        // Check arguments
                        for (i, param) in func.params.iter().enumerate() {
                            if let Some(actual_ty) = arg_types.get(i) {
                                let subst_expected =
                                    self.monomorphize_type(&param.ty, &type_mapping);
                                if !self.is_compatible(&subst_expected, actual_ty) {
                                    self.errors.push(SemaError::TypeMismatch(
                                        subst_expected,
                                        actual_ty.clone(),
                                        expr.span,
                                    ));
                                }
                            }
                        }

                        if args.len() != func.params.len() {
                            self.errors.push(SemaError::TypeMismatch(
                                Type::Void,
                                Type::Void,
                                expr.span,
                            )); // Using TypeMismatch as placeholder for WrongArgCount for now
                        }

                        return self.monomorphize_type(&func.return_type, &type_mapping);
                    } else {
                        let mut ext_opt = self.extern_fns.get(&resolved_name).cloned();
                        if ext_opt.is_none() && !self.current_package.is_empty() {
                            let temp_res = format!("{}::{}", self.current_package, name);
                            if self.extern_fns.contains_key(&temp_res) {
                                resolved_name = temp_res;
                                ext_opt = self.extern_fns.get(&resolved_name).cloned();
                            }
                        }
                        
                        if let Some(ext) = ext_opt {
                            self.resolved_names.insert(callee.span, resolved_name.clone());
                            for (i, param) in ext.params.iter().enumerate() {
                                if let Some(actual_ty) = arg_types.get(i) {
                                    if !self.is_compatible(&param.1, actual_ty) {
                                        self.errors.push(SemaError::TypeMismatch(
                                            param.1.clone(),
                                            actual_ty.clone(),
                                            expr.span,
                                        ));
                                    }
                                }
                            }
                            if args.len() != ext.params.len() {
                                self.errors.push(SemaError::TypeMismatch(Type::Void, Type::Void, expr.span));
                            }
                            return ext.return_type;
                        } else {
                            if let Some((en_name, var_name)) = name.split_once("::") {

                            if let Some(en) = self.enums.get(en_name).cloned() {
                                if en.variants.iter().any(|v| v.name == var_name) {
                                    return Type::Named(en_name.to_string(), vec![]);
                                }
                            }
                        }
                        self.errors.push(SemaError::UndefinedSymbol(name.clone(), expr.span));
                        }
                    }
                }
                Type::Void
            }
            ExprKind::MethodCall(base, method_name, args) => {
                let base_ty = self.analyze_expr(base);
                let mut arg_types = vec![base_ty.clone()];
                for arg in args {
                    arg_types.push(self.analyze_expr(arg));
                }

                let struct_name = match &base_ty {
                    Type::Named(name, _) => name.clone(),
                    Type::Ref(inner) | Type::RefMut(inner) => {
                        if let Type::Named(name, _) = &**inner {
                            name.clone()
                        } else {
                            "".to_string()
                        }
                    }
                    _ => "".to_string(),
                };

                let fn_name = format!("{}_{}", struct_name, method_name);
                if let Some(func) = self.functions.get(&fn_name).cloned() {
                    let mut type_mapping = HashMap::new();

                    for (i, param) in func.params.iter().enumerate() {
                        if let Some(actual_ty) = arg_types.get(i) {
                            Self::infer_generics(
                                &param.ty,
                                actual_ty,
                                &mut type_mapping,
                                &func.generics,
                            );
                        }
                    }

                    for (i, param) in func.params.iter().enumerate() {
                        if let Some(actual_ty) = arg_types.get(i) {
                            let subst_expected = self.monomorphize_type(&param.ty, &type_mapping);
                            if !self.is_compatible(&subst_expected, actual_ty) {
                                self.errors.push(SemaError::TypeMismatch(
                                    subst_expected,
                                    actual_ty.clone(),
                                    expr.span,
                                ));
                            }
                        }
                    }

                    if arg_types.len() != func.params.len() {
                        self.errors.push(SemaError::TypeMismatch(
                            Type::Void,
                            Type::Void,
                            expr.span,
                        ));
                    }

                    return self.monomorphize_type(&func.return_type, &type_mapping);
                } else {
                    self.errors
                        .push(SemaError::UndefinedSymbol(fn_name, expr.span));
                }
                Type::Void
            }
            ExprKind::If(cond, then_branch, else_branch) => {
                let _ct = self.analyze_expr(cond);
                let tt = self.analyze_expr(then_branch);
                if let Some(eb) = else_branch {
                    let _et = self.analyze_expr(eb);
                }
                tt
            }
            ExprKind::EnumVariant(enum_name, var_name, args) => {
                if let Some(en) = self.enums.get(enum_name).cloned() {
                    if let Some(var) = en.variants.iter().find(|v| v.name == *var_name) {
                        for (i, arg) in args.iter().enumerate() {
                            let arg_ty = self.analyze_expr(arg);
                            // TODO: check arg_ty vs var.types[i]
                        }
                    } else {
                        self.errors
                            .push(SemaError::UndefinedSymbol(var_name.clone(), expr.span));
                    }
                    Type::Named(enum_name.clone(), vec![]) // Naive generic resolution for MVP
                } else {
                    self.errors
                        .push(SemaError::UndefinedSymbol(enum_name.clone(), expr.span));
                    Type::Void
                }
            }
            ExprKind::Match(target, arms) => {
                let target_ty = self.analyze_expr(target);
                let mut res_ty = Type::Void;
                for (i, arm) in arms.iter().enumerate() {
                    self.enter_scope();

                    // Bind variables from pattern
                    match &arm.pattern {
                        apl_ast::Pattern::EnumVariant(en_name, var_name, sub_pats) => {
                            if let Some(en) = self.enums.get(en_name).cloned() {
                                if let Some(var) = en.variants.iter().find(|v| v.name == *var_name)
                                {
                                    for (j, sub_pat) in sub_pats.iter().enumerate() {
                                        if let apl_ast::Pattern::Ident(n) = sub_pat {
                                            let pat_ty =
                                                var.types.get(j).cloned().unwrap_or(Type::Void);
                                            // Assume it's a generic substitution to target_ty generic args, but for MVP we use pat_ty directly
                                            self.define_var(n.clone(), pat_ty, false, expr.span);
                                        }
                                    }
                                }
                            }
                        }
                        apl_ast::Pattern::Ident(n) => {
                            self.define_var(n.clone(), target_ty.clone(), false, expr.span);
                        }
                        _ => {}
                    }

                    if let Some(guard) = &arm.guard {
                        self.analyze_expr(guard);
                    }
                    let t = self.analyze_expr(&arm.body);
                    if i == 0 {
                        res_ty = t;
                    }
                    self.leave_scope();
                }
                res_ty
            }
            ExprKind::Block(stmts, trailing) => {
                self.enter_scope();
                for stmt in stmts {
                    match stmt {
                        Stmt::Let {
                            name,
                            mutability,
                            ty,
                            init,
                            span,
                        } => {
                            let init_ty = self.analyze_expr(init);
                            if let Some(expected_ty) = ty {
                                if !self.is_compatible(expected_ty, &init_ty) {
                                    self.errors.push(SemaError::TypeMismatch(
                                        expected_ty.clone(),
                                        init_ty.clone(),
                                        *span,
                                    ));
                                }
                            }
                            let var_ty = ty.clone().unwrap_or(init_ty);
                            let _ = self.define_var(name.clone(), var_ty, *mutability, *span);
                        }
                        Stmt::Expr(e) => {
                            self.analyze_expr(e);
                            // Reset borrows at the end of expression statements (naive NLL)
                            for scope in &mut self.scopes {
                                for var in scope.vars.values_mut() {
                                    if let OwnershipState::BorrowedMut(_)
                                    | OwnershipState::BorrowedImm(_) = var.ownership
                                    {
                                        var.ownership = OwnershipState::Owned;
                                    }
                                }
                            }
                        }
                    }
                }
                let res = if let Some(trailing_e) = trailing {
                    self.analyze_expr(trailing_e)
                } else {
                    Type::Void
                };
                self.leave_scope();
                res
            }
            ExprKind::Return(opt_val) => {
                let ret_ty = if let Some(val) = opt_val {
                    self.analyze_expr(val)
                } else {
                    Type::Void
                };
                if let Some(expected) = &self.current_fn_return_type {
                    if !self.is_compatible(expected, &ret_ty) {
                        self.errors.push(SemaError::TypeMismatch(
                            expected.clone(),
                            ret_ty.clone(),
                            expr.span,
                        ));
                    }
                }
                self.last_return_type = Some(ret_ty.clone());
                ret_ty
            }
            ExprKind::StructInit(name, fields) => {
                let mut resolved_name = name.clone();
                let mut decl_opt = self.structs.get(&resolved_name).cloned();
                if decl_opt.is_none() && !self.current_package.is_empty() {
                    resolved_name = format!("{}::{}", self.current_package, name);
                    decl_opt = self.structs.get(&resolved_name).cloned();
                }
                
                if let Some(decl) = decl_opt {
                    self.resolved_names.insert(expr.span, resolved_name.clone());

                    let mut provided_fields = HashMap::new();
                    for (f_name, f_expr) in fields {
                        let f_ty = self.analyze_expr(f_expr);
                        provided_fields.insert(f_name.clone(), f_ty);
                    }

                    let mut type_mapping = HashMap::new();

                    for expected_f in &decl.fields {
                        if let Some(actual_ty) = provided_fields.get(&expected_f.name) {
                            Self::infer_generics(
                                &expected_f.ty,
                                actual_ty,
                                &mut type_mapping,
                                &decl.generics,
                            );
                        }
                    }

                    for expected_f in &decl.fields {
                        if let Some(actual_ty) = provided_fields.remove(&expected_f.name) {
                            let subst_expected =
                                self.monomorphize_type(&expected_f.ty, &type_mapping);
                            if !self.is_compatible(&subst_expected, &actual_ty) {
                                self.errors.push(SemaError::TypeMismatch(
                                    subst_expected,
                                    actual_ty,
                                    expr.span,
                                ));
                            }
                        } else {
                            self.errors.push(SemaError::MissingField(
                                name.clone(),
                                expected_f.name.clone(),
                                expr.span,
                            ));
                        }
                    }

                    for (extra_f, _) in provided_fields {
                        self.errors
                            .push(SemaError::UnknownField(name.clone(), extra_f, expr.span));
                    }

                    let resolved_args = decl
                        .generics
                        .iter()
                        .map(|g| {
                            type_mapping
                                .get(g)
                                .cloned()
                                .unwrap_or(Type::Named(g.clone(), vec![]))
                        })
                        .collect();
                    Type::Named(name.clone(), resolved_args)
                } else {
                    self.errors
                        .push(SemaError::UndefinedSymbol(name.clone(), expr.span));
                    Type::Void
                }
            }
            ExprKind::FieldAccess(base_expr, field_name) => {
                let prev = self.is_borrow_context;
                self.is_borrow_context = true;
                let base_ty = self.analyze_expr(base_expr);
                self.is_borrow_context = prev;

                match base_ty {
                    Type::Named(name, args) => {
                        if let Some(decl) = self.structs.get(&name).cloned() {
                            if let Some(f) = decl.fields.iter().find(|f| f.name == *field_name) {
                                let mut type_mapping = HashMap::new();
                                for (i, g) in decl.generics.iter().enumerate() {
                                    if let Some(arg) = args.get(i) {
                                        type_mapping.insert(g.clone(), arg.clone());
                                    }
                                }
                                self.monomorphize_type(&f.ty, &type_mapping)
                            } else {
                                self.errors.push(SemaError::UnknownField(
                                    name.clone(),
                                    field_name.clone(),
                                    expr.span,
                                ));
                                Type::Void
                            }
                        } else {
                            Type::Void
                        }
                    }
                    Type::Ref(inner) | Type::RefMut(inner) => {
                        if let Type::Named(name, args) = *inner {
                            if let Some(decl) = self.structs.get(&name).cloned() {
                                if let Some(f) = decl.fields.iter().find(|f| f.name == *field_name)
                                {
                                    let mut type_mapping = HashMap::new();
                                    for (i, g) in decl.generics.iter().enumerate() {
                                        if let Some(arg) = args.get(i) {
                                            type_mapping.insert(g.clone(), arg.clone());
                                        }
                                    }
                                    self.monomorphize_type(&f.ty, &type_mapping)
                                } else {
                                    self.errors.push(SemaError::UnknownField(
                                        name.clone(),
                                        field_name.clone(),
                                        expr.span,
                                    ));
                                    Type::Void
                                }
                            } else {
                                Type::Void
                            }
                        } else {
                            Type::Void // Error reporting omitted for brevity
                        }
                    }
                    _ => {
                        // Not a struct
                        Type::Void
                    }
                }
            }
            ExprKind::While(cond, body) => {
                let cond_ty = self.analyze_expr(cond);
                if cond_ty != Type::Bool {
                    self.errors
                        .push(SemaError::TypeMismatch(Type::Bool, cond_ty, cond.span));
                }
                self.loop_depth += 1;
                self.analyze_expr(body);
                self.loop_depth -= 1;
                Type::Void
            }
            ExprKind::Loop(body) => {
                self.loop_depth += 1;
                self.analyze_expr(body);
                self.loop_depth -= 1;
                Type::Void
            }
            ExprKind::Break | ExprKind::Continue => {
                if self.loop_depth == 0 {
                    self.errors.push(SemaError::BreakOutsideLoop(expr.span));
                }
                Type::Void
            }
            ExprKind::ArrayInit(elements) => {
                if elements.is_empty() {
                    Type::Array(Box::new(Type::Void), 0) // Should ideally infer type, but YKL needs help
                } else {
                    let first_ty = self.analyze_expr(&elements[0]);
                    for el in elements.iter().skip(1) {
                        let ty = self.analyze_expr(el);
                        if !self.is_compatible(&first_ty, &ty) {
                            self.errors.push(SemaError::TypeMismatch(
                                first_ty.clone(),
                                ty,
                                el.span,
                            ));
                        }
                    }
                    Type::Array(Box::new(first_ty), elements.len())
                }
            }
            ExprKind::Index(base, index) => {
                let prev = self.is_borrow_context;
                self.is_borrow_context = true;
                let base_ty = self.analyze_expr(base);
                self.is_borrow_context = prev;

                let index_ty = self.analyze_expr(index);
                if index_ty != Type::I32 && index_ty != Type::I64 && index_ty != Type::USize {
                    self.errors
                        .push(SemaError::TypeMismatch(Type::USize, index_ty, index.span));
                }

                match base_ty {
                    Type::Array(inner, _) | Type::Slice(inner) => *inner,
                    Type::Ref(inner) | Type::RefMut(inner) => match *inner {
                        Type::Array(t, _) | Type::Slice(t) => *t,
                        _ => Type::Void,
                    },
                    _ => Type::Void,
                }
            }
            _ => Type::Void,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apl_lexer::Lexer;
    use apl_parser::Parser;

    #[test]
    fn test_sema_borrow_checker() {
        let input = "fn main() { let x = 10; let y = x; let z = x; }";
        let mut lexer = Lexer::new(input, 0);
        let mut parser = Parser::new(lexer.tokenize().unwrap());
        let prog = parser.parse_program();
        let mut sema = SemanticAnalyzer::new();
        assert!(sema.analyze_program(&prog).is_ok());
    }

    #[test]
    fn test_sema_move_error() {
        let input = "fn main() { let s = \"hello\"; let s2 = s; let s3 = s; }";
        let mut lexer = Lexer::new(input, 0);
        let mut parser = Parser::new(lexer.tokenize().unwrap());
        let prog = parser.parse_program();
        let mut sema = SemanticAnalyzer::new();
        assert!(sema.analyze_program(&prog).is_err());
    }
}
