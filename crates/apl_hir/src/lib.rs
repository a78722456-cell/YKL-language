
pub mod lowering;
pub mod printer;

use apl_ast::{Span, Type, BinaryOp, UnaryOp};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnumId(pub u32);

#[derive(Debug, Clone)]
pub struct HirProgram {
    pub functions: Vec<HirFunction>,
    pub structs: Vec<HirStruct>,
    pub enums: Vec<HirEnum>,
}

#[derive(Debug, Clone)]
pub struct HirStruct {
    pub id: StructId,
    pub name: String,
    pub fields: Vec<HirField>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirField {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct HirEnum {
    pub id: EnumId,
    pub name: String,
    pub variants: Vec<HirVariant>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirVariant {
    pub name: String,
    pub types: Vec<Type>,
}

#[derive(Debug, Clone)]
pub struct HirFunction {
    pub id: FunctionId,
    pub name: String, // Fully resolved (e.g. `std::vec::Vec_push`)
    pub params: Vec<HirParam>,
    pub return_type: Type,
    pub body: Option<HirBlock>, // None if extern
    pub is_extern: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirParam {
    pub id: LocalId,
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirBlock {
    pub stmts: Vec<HirStmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum HirStmt {
    Let {
        id: LocalId,
        name: String,
        mutability: bool,
        ty: Type,
        init: Option<HirExpr>,
        span: Span,
    },
    Expr(HirExpr),
    Return(Option<HirExpr>, Span),
}

#[derive(Debug, Clone)]
pub struct HirExpr {
    pub kind: HirExprKind,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirMatchArm {
    pub variant_tag: Option<u32>,
    pub payload_vars: Vec<(LocalId, Type)>,
    pub body: HirExpr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum HirExprKind {
    LiteralInt(i64),
    LiteralFloat(f64),
    LiteralBool(bool),
    LiteralString(String),
    LocalRef(LocalId), // Replaced Var(String)
    GlobalRef(String), // Or GlobalId
    Call {
        callee: FunctionId, // Replaced String
        args: Vec<HirExpr>,
    },
    MethodCall {
        receiver: Box<HirExpr>,
        method: FunctionId,
        args: Vec<HirExpr>,
    },
    StructInit {
        struct_id: StructId,
        fields: Vec<(String, HirExpr)>,
    },
    FieldAccess(Box<HirExpr>, String), // Will become FieldId
    Index(Box<HirExpr>, Box<HirExpr>),
    ArrayInit(Vec<HirExpr>),
    Match {
        scrutinee: Box<HirExpr>,
        arms: Vec<HirMatchArm>,
    },
    EnumInit {
        enum_name: String,
        tag: u32,
        args: Vec<HirExpr>,
    },
    BinaryOp(BinaryOp, Box<HirExpr>, Box<HirExpr>),
    UnaryOp(UnaryOp, Box<HirExpr>),
    Assign(Box<HirExpr>, Box<HirExpr>),
    If {
        cond: Box<HirExpr>,
        then_branch: Box<HirExpr>,
        else_branch: Option<Box<HirExpr>>,
    },
    While {
        cond: Box<HirExpr>,
        body: Box<HirExpr>,
    },
    Block(HirBlock, Option<Box<HirExpr>>),
    Loop(Box<HirExpr>),
    Break,
    Continue,
    Return(Option<Box<HirExpr>>),
}


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
            HirExprKind::Call { callee: _, args } => {
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


