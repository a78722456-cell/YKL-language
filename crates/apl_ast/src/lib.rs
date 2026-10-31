use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    pub file_id: usize,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub col: usize,
}

impl Span {
    pub fn new(file_id: usize, start: usize, end: usize, line: usize, col: usize) -> Self {
        Self {
            file_id,
            start,
            end,
            line,
            col,
        }
    }
    pub fn dummy() -> Self {
        Self {
            file_id: 0, start: 0,
            end: 0,
            line: 1,
            col: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    I8,
    I16,
    I32,
    I64,
    ISize,
    U8,
    U16,
    U32,
    U64,
    USize,
    F32,
    F64,
    Bool,
    Char,
    String,
    Void,
    Named(String, Vec<Type>),
    Ref(Box<Type>),
    RefMut(Box<Type>),
    Nullable(Box<Type>),
    Ptr, // Raw pointer (e.g. for C FFI)
    Array(Box<Type>, usize),
    Slice(Box<Type>),
    Channel(Box<Type>),
}

impl Type {
    pub fn is_copy(&self) -> bool {
        match self {
            Type::Named(..) | Type::String | Type::Array(..) | Type::Slice(..) | Type::Channel(..) => false,
            _ => true,
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::I8 => write!(f, "i8"),
            Type::I16 => write!(f, "i16"),
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
            Type::ISize => write!(f, "isize"),
            Type::U8 => write!(f, "u8"),
            Type::U16 => write!(f, "u16"),
            Type::U32 => write!(f, "u32"),
            Type::U64 => write!(f, "u64"),
            Type::USize => write!(f, "usize"),
            Type::F32 => write!(f, "f32"),
            Type::F64 => write!(f, "f64"),
            Type::Bool => write!(f, "bool"),
            Type::Char => write!(f, "char"),
            Type::String => write!(f, "string"),
            Type::Void => write!(f, "void"),
            Type::Ptr => write!(f, "Ptr"),
            Type::Named(name, args) => {
                write!(f, "{}", name)?;
                if !args.is_empty() {
                    write!(f, "<")?;
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 { write!(f, ", ")?; }
                        write!(f, "{}", arg)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            },
            Type::Ref(t) => write!(f, "&{}", t),
            Type::RefMut(t) => write!(f, "&mut {}", t),
            Type::Nullable(t) => write!(f, "?{}", t),
            Type::Array(t, len) => write!(f, "[{}; {}]", t, len),
            Type::Slice(t) => write!(f, "[{}]", t),
            Type::Channel(t) => write!(f, "chan {}", t),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Char(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg,
    Not,
    Ref,
    RefMut,
    Deref,
    TryUnwrap,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    Ident(String),
    Literal(Literal),
    Wildcard,
    EnumVariant(String, String, Vec<Pattern>), // EnumName, VariantName, SubPatterns
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExprKind {
    Literal(Literal),
    Ident(String),
    Assign(Box<Expr>, Box<Expr>),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    MethodCall(Box<Expr>, String, Vec<Expr>),
    FieldAccess(Box<Expr>, String),
    Index(Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    Match(Box<Expr>, Vec<MatchArm>),
    EnumVariant(String, String, Vec<Expr>),
    Block(Vec<Stmt>, Option<Box<Expr>>),
    Return(Option<Box<Expr>>),
    Break,
    Continue,
    Await(Box<Expr>),
    Spawn(Box<Expr>),
    Loop(Box<Expr>),
    While(Box<Expr>, Box<Expr>),
    StructInit(String, Vec<(String, Expr)>),
    ArrayInit(Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Stmt {
    Let {
        name: String,
        mutability: bool,
        ty: Option<Type>,
        init: Expr,
        span: Span,
    },
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub generics: Vec<String>,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub is_async: bool,
    pub is_pub: bool,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
    pub is_pub: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructDecl {
    pub name: String,
    pub generics: Vec<String>,
    pub fields: Vec<StructField>,
    pub is_pub: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariantDecl {
    pub name: String,
    pub types: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumDecl {
    pub name: String,
    pub generics: Vec<String>,
    pub variants: Vec<EnumVariantDecl>,
    pub is_pub: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitDecl {
    pub name: String,
    pub methods: Vec<Function>,
    pub is_pub: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImplDecl {
    pub target_type: String,
    pub trait_name: Option<String>,
    pub methods: Vec<Function>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternDecl {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Item {
    Package(String, Span),
    Import(String, Span),
    Function(Function),
    Struct(StructDecl),
    Enum(EnumDecl),
    Trait(TraitDecl),
    Impl(ImplDecl),
    ExternBlock(Vec<ExternDecl>, Span),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub items: Vec<Item>,
}
