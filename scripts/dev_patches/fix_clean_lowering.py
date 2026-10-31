with open('crates/apl_hir/src/lowering.rs', 'w', encoding='utf-8') as f:
    f.write('''use crate::*;
use apl_ast::{Expr, ExprKind, Item, Program, Span, Stmt, Type};
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
}

impl HirLowering {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            scopes: Vec::new(),
            functions: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            current_package: String::new(),
            current_fn_return_type: None,
            resolved_names: HashMap::new(),
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn leave_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn define_var(&mut self, name: String, ty: Type, mutability: bool, span: Span) -> Result<(), ()> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.vars.contains_key(&name) {
                self.errors.push(HirError::DuplicateSymbol(name, span));
                return Err(());
            }
            scope.vars.insert(
                name.clone(),
                VarInfo {
                    name,
                    ty,
                    mutability,
                    span,
                },
            );
        }
        Ok(())
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
        
        // Check if extern
        // If not extern, body is always there
        // Our parser might not have Option<Expr> for func.body, wait, func.body is Expr
        let body = if let ExprKind::Block(stmts, _opt_trailing) = &func.body.kind {
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
                // ty is Option<Type> in apl_ast
                let defined_ty = ty.clone().unwrap_or(Type::Void);
                let hir_init = if let Some(e) = init {
                    Some(self.lower_expr(e))
                } else {
                    None
                };
                let _ = self.define_var(name.clone(), defined_ty.clone(), *mutability, *span);
                Some(HirStmt::Let {
                    name: name.clone(),
                    mutability: *mutability,
                    ty: defined_ty,
                    init: hir_init,
                    span: *span,
                })
            }
            Stmt::Expr(expr) => {
                Some(HirStmt::Expr(self.lower_expr(expr)))
            }
            // we will need to handle Returns later (in apl_ast, return is an Expr)
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
