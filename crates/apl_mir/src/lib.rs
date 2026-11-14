
pub use apl_hir::{LocalId, FunctionId};
use apl_ast::{Span, Type};
pub mod cfg;
pub mod printer;
pub mod lowering;
pub mod dataflow;
pub mod validate;
pub mod analysis;
pub mod interpret;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BasicBlockId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FieldId(pub u32);

#[derive(Debug, Clone)]
pub struct MirBody {
    pub locals: Vec<LocalDecl>,
    pub blocks: Vec<BasicBlock>,
    pub start_block: BasicBlockId,
}

#[derive(Debug, Clone)]
pub struct LocalDecl {
    pub id: LocalId,
    pub ty: Type,
    pub span: Span,
    pub mutability: bool,
    pub is_param: bool,
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub statements: Vec<Statement>,
    pub terminator: Terminator,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Assign(Place, Rvalue, Span),
    StorageLive(LocalId, Span),
    StorageDead(LocalId, Span),
    Drop(Place, Span),
    Nop,
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Goto(BasicBlockId),
    SwitchInt {
        discriminant: Operand,
        targets: Vec<(u128, BasicBlockId)>,
        otherwise: BasicBlockId,
    },
    Call {
        function: FunctionId, // simplified for now
        args: Vec<Operand>,
        destination: Place,
        target: BasicBlockId,
        span: Span,
    },
    Return(Span),
    Unreachable,
    Drop {
        place: Place,
        target: BasicBlockId,
        span: Span,
    },
    Resume, // for unwinding
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Place {
    Local(LocalId),
    Field {
        base: Box<Place>,
        field: FieldId,
    },
    Index {
        base: Box<Place>,
        index: LocalId,
    },
    Deref {
        base: Box<Place>,
    },
}

#[derive(Debug, Clone)]
pub enum Rvalue {
    Use(Operand),
    Ref(Region, BorrowKind, Place),
    BinaryOp(apl_ast::BinaryOp, Operand, Operand),
    UnaryOp(apl_ast::UnaryOp, Operand),
    Aggregate(Vec<Operand>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Region(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BorrowKind {
    Shared,
    Mut,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Copy(Place),
    Move(Place),
    Constant(Constant),
}

#[derive(Debug, Clone)]
pub enum Constant {
    Int(i64, Type),
    Float(f64, Type),
    Bool(bool),
    String(String),
}

