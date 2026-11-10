/// MIR Interpreter — executes MirBody directly without generating C or bytecode.
/// This enables end-to-end `run` without a C compiler backend.

use crate::*;
use apl_ast::{BinaryOp, UnaryOp, Type};
use apl_hir::{LocalId, FunctionId};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Aggregate(Vec<Value>),
    Ref(usize), // heap address for references
    Null,
    Void,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Str(s) => write!(f, "{}", s),
            Value::Aggregate(fields) => {
                write!(f, "{{")?;
                for (i, v) in fields.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", v)?;
                }
                write!(f, "}}")
            }
            Value::Ref(addr) => write!(f, "&{}", addr),
            Value::Null => write!(f, "null"),
            Value::Void => write!(f, "()"),
        }
    }
}

#[derive(Debug)]
pub enum InterpretError {
    UndefinedLocal(LocalId),
    UndefinedFunction(FunctionId),
    DivisionByZero,
    TypeMismatch(String),
    UnreachableReached,
    ReturnOutsideFunction,
    StackOverflow,
    InvalidMemoryAccess(usize),
    UserDefined(String),
}

impl std::fmt::Display for InterpretError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpretError::UndefinedLocal(id) => write!(f, "undefined local _{}", id.0),
            InterpretError::UndefinedFunction(id) => write!(f, "undefined function {:?}", id),
            InterpretError::DivisionByZero => write!(f, "division by zero"),
            InterpretError::TypeMismatch(s) => write!(f, "type mismatch: {}", s),
            InterpretError::UnreachableReached => write!(f, "unreachable code reached"),
            InterpretError::ReturnOutsideFunction => write!(f, "return outside function"),
            InterpretError::StackOverflow => write!(f, "stack overflow"),
            InterpretError::InvalidMemoryAccess(addr) => write!(f, "invalid memory access at {}", addr),
            InterpretError::UserDefined(msg) => write!(f, "runtime error: {}", msg),
        }
    }
}

/// Map: function_id → body (for native YKL functions)
type FuncMap = HashMap<FunctionId, MirBody>;
/// Map: function_name → function_id
type NameMap = HashMap<String, FunctionId>;

pub struct Interpreter {
    functions: FuncMap,
    function_names: NameMap,
    heap: Vec<Value>,
    pub output: Vec<String>,
    call_depth: usize,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            function_names: HashMap::new(),
            heap: Vec::new(),
            output: Vec::new(),
            call_depth: 0,
        }
    }

    pub fn register_function(&mut self, id: FunctionId, name: impl Into<String>, body: MirBody) {
        let name = name.into();
        self.function_names.insert(name, id);
        self.functions.insert(id, body);
    }

    pub fn call_by_name(&mut self, name: &str, args: Vec<Value>) -> Result<Value, InterpretError> {
        let fid = self.function_names.get(name).copied()
            .ok_or_else(|| InterpretError::UserDefined(format!("function '{}' not found", name)))?;
        self.call(fid, args)
    }

    pub fn call(&mut self, func_id: FunctionId, args: Vec<Value>) -> Result<Value, InterpretError> {
        if self.call_depth > 256 {
            return Err(InterpretError::StackOverflow);
        }
        if !self.functions.contains_key(&func_id) {
            return Err(InterpretError::UndefinedFunction(func_id));
        }

        self.call_depth += 1;
        let body = self.functions[&func_id].clone();

        // Frame: local_id → value
        let mut frame: HashMap<LocalId, Value> = HashMap::new();
        let mut param_idx = 0usize;
        for local in &body.locals {
            if local.is_param {
                let v = args.get(param_idx).cloned().unwrap_or(Value::Void);
                frame.insert(local.id, v);
                param_idx += 1;
            } else {
                frame.insert(local.id, Value::Void);
            }
        }

        let result = self.execute_body(&body, &mut frame);
        self.call_depth -= 1;
        result
    }

    fn execute_body(&mut self, body: &MirBody, frame: &mut HashMap<LocalId, Value>) -> Result<Value, InterpretError> {
        let mut current_block = body.start_block;

        loop {
            let block = body.blocks[current_block.0 as usize].clone();

            for stmt in &block.statements {
                match stmt {
                    Statement::Assign(place, rvalue, _) => {
                        let val = self.eval_rvalue(rvalue, frame)?;
                        self.write_place(place, val, frame)?;
                    }
                    Statement::StorageLive(id, _) => {
                        frame.entry(*id).or_insert(Value::Void);
                    }
                    Statement::StorageDead(id, _) => {
                        frame.insert(*id, Value::Void);
                    }
                    Statement::Drop(Place::Local(id), _) => {
                        frame.insert(*id, Value::Void);
                    }
                    Statement::Drop(_, _) => {}
                    Statement::Nop => {}
                }
            }

            match block.terminator {
                Terminator::Goto(target) => {
                    current_block = target;
                }
                Terminator::Return(_) => {
                    let ret_val = frame.get(&LocalId(0)).cloned().unwrap_or(Value::Void);
                    return Ok(ret_val);
                }
                Terminator::SwitchInt { discriminant, targets, otherwise } => {
                    let val = self.eval_operand(&discriminant, frame)?;
                    let disc_int = match val {
                        Value::Int(n) => n as u128,
                        Value::Bool(b) => if b { 1 } else { 0 },
                        _ => return Err(InterpretError::TypeMismatch(
                            format!("SwitchInt on non-integer: {:?}", val)
                        )),
                    };
                    let mut jumped = false;
                    for (case, target) in &targets {
                        if disc_int == *case {
                            current_block = *target;
                            jumped = true;
                            break;
                        }
                    }
                    if !jumped {
                        current_block = otherwise;
                    }
                }
                Terminator::Call { function, args, destination, target, .. } => {
                    let mut arg_vals = Vec::new();
                    for arg in &args {
                        arg_vals.push(self.eval_operand(arg, frame)?);
                    }
                    let ret = self.call(function, arg_vals)?;
                    self.write_place(&destination, ret, frame)?;
                    current_block = target;
                }
                Terminator::Drop { place, target, .. } => {
                    if let Place::Local(id) = &place {
                        frame.insert(*id, Value::Void);
                    }
                    current_block = target;
                }
                Terminator::Unreachable => {
                    return Err(InterpretError::UnreachableReached);
                }
                Terminator::Resume => {
                    return Err(InterpretError::UserDefined("unwind resume".to_string()));
                }
            }
        }
    }

    fn eval_rvalue(&mut self, rvalue: &Rvalue, frame: &mut HashMap<LocalId, Value>) -> Result<Value, InterpretError> {
        match rvalue {
            Rvalue::Use(op) => self.eval_operand(op, frame),
            Rvalue::Ref(_, _kind, place) => {
                let val = self.read_place(place, frame)?;
                let addr = self.heap.len();
                self.heap.push(val);
                Ok(Value::Ref(addr))
            }
            Rvalue::BinaryOp(op, l, r) => {
                let lv = self.eval_operand(l, frame)?;
                let rv = self.eval_operand(r, frame)?;
                Self::eval_binop(*op, lv, rv)
            }
            Rvalue::UnaryOp(op, operand) => {
                let val = self.eval_operand(operand, frame)?;
                Self::eval_unop(*op, val)
            }
            Rvalue::Aggregate(ops) => {
                let mut fields = Vec::new();
                for op in ops {
                    fields.push(self.eval_operand(op, frame)?);
                }
                Ok(Value::Aggregate(fields))
            }
        }
    }

    fn eval_operand(&mut self, op: &Operand, frame: &mut HashMap<LocalId, Value>) -> Result<Value, InterpretError> {
        match op {
            Operand::Copy(place) | Operand::Move(place) => self.read_place(place, frame),
            Operand::Constant(c) => Ok(match c {
                Constant::Int(n, _)  => Value::Int(*n),
                Constant::Float(f, _) => Value::Float(*f),
                Constant::Bool(b)    => Value::Bool(*b),
                Constant::String(s)  => Value::Str(s.clone()),
            }),
        }
    }

    fn read_place(&self, place: &Place, frame: &HashMap<LocalId, Value>) -> Result<Value, InterpretError> {
        match place {
            Place::Local(id) => {
                frame.get(id).cloned().ok_or(InterpretError::UndefinedLocal(*id))
            }
            Place::Field { base, field } => {
                let base_val = self.read_place(base, frame)?;
                match base_val {
                    Value::Aggregate(fields) => {
                        let idx = field.0 as usize;
                        fields.get(idx).cloned().ok_or_else(|| {
                            InterpretError::TypeMismatch(format!("field {} out of bounds", idx))
                        })
                    }
                    _ => Err(InterpretError::TypeMismatch("field access on non-aggregate".to_string())),
                }
            }
            Place::Deref { base } => {
                let base_val = self.read_place(base, frame)?;
                match base_val {
                    Value::Ref(addr) => {
                        self.heap.get(addr).cloned().ok_or(InterpretError::InvalidMemoryAccess(addr))
                    }
                    _ => Err(InterpretError::TypeMismatch("deref on non-reference".to_string())),
                }
            }
            Place::Index { base, index } => {
                let base_val = self.read_place(base, frame)?;
                let idx_val = frame.get(index).cloned().ok_or(InterpretError::UndefinedLocal(*index))?;
                match (base_val, idx_val) {
                    (Value::Aggregate(fields), Value::Int(i)) => {
                        fields.get(i as usize).cloned().ok_or_else(|| {
                            InterpretError::TypeMismatch(format!("index {} out of bounds", i))
                        })
                    }
                    _ => Err(InterpretError::TypeMismatch("index on non-array".to_string())),
                }
            }
        }
    }

    fn write_place(&mut self, place: &Place, val: Value, frame: &mut HashMap<LocalId, Value>) -> Result<(), InterpretError> {
        match place {
            Place::Local(id) => {
                frame.insert(*id, val);
                Ok(())
            }
            Place::Field { base, field } => {
                let base_val = self.read_place(base, frame)?;
                match base_val {
                    Value::Aggregate(mut fields) => {
                        let idx = field.0 as usize;
                        if idx < fields.len() {
                            fields[idx] = val;
                            self.write_place(base, Value::Aggregate(fields), frame)
                        } else {
                            Err(InterpretError::TypeMismatch(format!("field {} out of bounds", idx)))
                        }
                    }
                    _ => Err(InterpretError::TypeMismatch("field write on non-aggregate".to_string())),
                }
            }
            Place::Deref { base } => {
                let base_val = self.read_place(base, frame)?;
                match base_val {
                    Value::Ref(addr) => {
                        if addr < self.heap.len() {
                            self.heap[addr] = val;
                            Ok(())
                        } else {
                            Err(InterpretError::InvalidMemoryAccess(addr))
                        }
                    }
                    _ => Err(InterpretError::TypeMismatch("deref write on non-reference".to_string())),
                }
            }
            Place::Index { base, index } => {
                let idx_val = frame.get(index).cloned().ok_or(InterpretError::UndefinedLocal(*index))?;
                let base_val = self.read_place(base, frame)?;
                match (base_val, idx_val) {
                    (Value::Aggregate(mut fields), Value::Int(i)) => {
                        let idx = i as usize;
                        if idx < fields.len() {
                            fields[idx] = val;
                            self.write_place(base, Value::Aggregate(fields), frame)
                        } else {
                            Err(InterpretError::TypeMismatch(format!("index {} out of bounds", i)))
                        }
                    }
                    _ => Err(InterpretError::TypeMismatch("index write on non-array".to_string())),
                }
            }
        }
    }

    fn eval_binop(op: BinaryOp, l: Value, r: Value) -> Result<Value, InterpretError> {
        // NOTE: variant names must match apl_ast::BinaryOp exactly
        match (op, l, r) {
            // Integer arithmetic
            (BinaryOp::Add, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_add(b))),
            (BinaryOp::Sub, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_sub(b))),
            (BinaryOp::Mul, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_mul(b))),
            (BinaryOp::Div, Value::Int(a), Value::Int(b)) => {
                if b == 0 { Err(InterpretError::DivisionByZero) } else { Ok(Value::Int(a / b)) }
            }
            (BinaryOp::Mod, Value::Int(a), Value::Int(b)) => {
                if b == 0 { Err(InterpretError::DivisionByZero) } else { Ok(Value::Int(a % b)) }
            }
            // Float arithmetic
            (BinaryOp::Add, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (BinaryOp::Sub, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (BinaryOp::Mul, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (BinaryOp::Div, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
            // Comparisons — integer (use actual AST variant names: Eq, Neq, Lt, Lte, Gt, Gte)
            (BinaryOp::Eq,  Value::Int(a), Value::Int(b))  => Ok(Value::Bool(a == b)),
            (BinaryOp::Neq, Value::Int(a), Value::Int(b))  => Ok(Value::Bool(a != b)),
            (BinaryOp::Lt,  Value::Int(a), Value::Int(b))  => Ok(Value::Bool(a < b)),
            (BinaryOp::Lte, Value::Int(a), Value::Int(b))  => Ok(Value::Bool(a <= b)),
            (BinaryOp::Gt,  Value::Int(a), Value::Int(b))  => Ok(Value::Bool(a > b)),
            (BinaryOp::Gte, Value::Int(a), Value::Int(b))  => Ok(Value::Bool(a >= b)),
            // Comparisons — float
            (BinaryOp::Eq,  Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a == b)),
            (BinaryOp::Neq, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a != b)),
            (BinaryOp::Lt,  Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
            (BinaryOp::Lte, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
            (BinaryOp::Gt,  Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
            (BinaryOp::Gte, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
            // Bool logic
            (BinaryOp::And, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
            (BinaryOp::Or,  Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
            // String concat
            (BinaryOp::Add, Value::Str(a), Value::Str(b)) => Ok(Value::Str(a + &b)),
            // Comparisons — bool
            (BinaryOp::Eq,  Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a == b)),
            (BinaryOp::Neq, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a != b)),
            (op, l, r) => Err(InterpretError::TypeMismatch(
                format!("unsupported binop {:?} on {:?} and {:?}", op, l, r)
            )),
        }
    }

    fn eval_unop(op: UnaryOp, val: Value) -> Result<Value, InterpretError> {
        match (op, val) {
            (UnaryOp::Neg, Value::Int(n))   => Ok(Value::Int(-n)),
            (UnaryOp::Neg, Value::Float(f)) => Ok(Value::Float(-f)),
            (UnaryOp::Not, Value::Bool(b))  => Ok(Value::Bool(!b)),
            // Ref/RefMut produce a heap address — simplified here (Rvalue::Ref handles proper refs)
            (UnaryOp::Ref, val)    => Ok(Value::Ref(0)),
            (UnaryOp::RefMut, val) => Ok(Value::Ref(0)),
            (UnaryOp::Deref, Value::Ref(_)) => {
                // actual deref via Place::Deref, here just return Void as placeholder
                Ok(Value::Void)
            }
            (UnaryOp::TryUnwrap, val) => Ok(val),
            (op, val) => Err(InterpretError::TypeMismatch(
                format!("unsupported unop {:?} on {:?}", op, val)
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apl_hir::LocalId;
    use apl_ast::{Type, Span};

    fn sp() -> Span {
        Span { file_id: 0, start: 0, end: 0, line: 0, col: 0 }
    }

    fn make_return_const(n: i64) -> MirBody {
        let sp = sp();
        let locals = vec![
            LocalDecl { id: LocalId(0), ty: Type::I32, span: sp, mutability: false, is_param: false },
        ];
        let block = BasicBlock {
            statements: vec![
                Statement::Assign(
                    Place::Local(LocalId(0)),
                    Rvalue::Use(Operand::Constant(Constant::Int(n, Type::I32))),
                    sp,
                ),
            ],
            terminator: Terminator::Return(sp),
        };
        MirBody { locals, blocks: vec![block], start_block: BasicBlockId(0) }
    }

    #[test]
    fn test_return_constant() {
        let body = make_return_const(42);
        let fid = FunctionId(0);
        let mut interp = Interpreter::new();
        interp.register_function(fid, "main", body);
        let result = interp.call_by_name("main", vec![]).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_simple_add() {
        let sp = sp();
        let r = Type::I32;

        let locals = vec![
            LocalDecl { id: LocalId(0),    ty: r.clone(), span: sp, mutability: false, is_param: false },
            LocalDecl { id: LocalId(1),    ty: r.clone(), span: sp, mutability: false, is_param: false },
            LocalDecl { id: LocalId(2),    ty: r.clone(), span: sp, mutability: false, is_param: false },
            LocalDecl { id: LocalId(1000), ty: r.clone(), span: sp, mutability: false, is_param: false },
        ];

        let block = BasicBlock {
            statements: vec![
                Statement::StorageLive(LocalId(1), sp),
                Statement::Assign(Place::Local(LocalId(1)), Rvalue::Use(Operand::Constant(Constant::Int(1, Type::I32))), sp),
                Statement::StorageLive(LocalId(2), sp),
                Statement::Assign(Place::Local(LocalId(2)), Rvalue::Use(Operand::Constant(Constant::Int(2, Type::I32))), sp),
                Statement::StorageLive(LocalId(1000), sp),
                Statement::Assign(
                    Place::Local(LocalId(1000)),
                    Rvalue::BinaryOp(BinaryOp::Add, Operand::Copy(Place::Local(LocalId(1))), Operand::Copy(Place::Local(LocalId(2)))),
                    sp,
                ),
                Statement::Assign(Place::Local(LocalId(0)), Rvalue::Use(Operand::Copy(Place::Local(LocalId(1000)))), sp),
            ],
            terminator: Terminator::Return(sp),
        };

        let body = MirBody { locals, blocks: vec![block], start_block: BasicBlockId(0) };
        let fid = FunctionId(0);
        let mut interp = Interpreter::new();
        interp.register_function(fid, "main", body);
        let result = interp.call_by_name("main", vec![]).unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_if_branch() {
        // if 1 == 1 { return 10; } else { return 20; }
        let sp = sp();

        // bb0: SwitchInt(1 == 1) -> then=bb1, else=bb2
        // bb1: _0 = 10; goto bb3
        // bb2: _0 = 20; goto bb3
        // bb3: return

        let locals = vec![
            LocalDecl { id: LocalId(0), ty: Type::I32, span: sp, mutability: false, is_param: false },
            LocalDecl { id: LocalId(1000), ty: Type::Bool, span: sp, mutability: false, is_param: false },
            LocalDecl { id: LocalId(1001), ty: Type::I32, span: sp, mutability: false, is_param: false },
        ];

        let bb0 = BasicBlock {
            statements: vec![
                Statement::StorageLive(LocalId(1000), sp),
                Statement::Assign(
                    Place::Local(LocalId(1000)),
                    Rvalue::BinaryOp(BinaryOp::Eq, Operand::Constant(Constant::Int(1, Type::I32)), Operand::Constant(Constant::Int(1, Type::I32))),
                    sp,
                ),
            ],
            terminator: Terminator::SwitchInt {
                discriminant: Operand::Copy(Place::Local(LocalId(1000))),
                targets: vec![(1, BasicBlockId(1))],
                otherwise: BasicBlockId(2),
            },
        };
        let bb1 = BasicBlock {
            statements: vec![
                Statement::StorageLive(LocalId(1001), sp),
                Statement::Assign(Place::Local(LocalId(1001)), Rvalue::Use(Operand::Constant(Constant::Int(10, Type::I32))), sp),
                Statement::Assign(Place::Local(LocalId(0)), Rvalue::Use(Operand::Copy(Place::Local(LocalId(1001)))), sp),
            ],
            terminator: Terminator::Goto(BasicBlockId(3)),
        };
        let bb2 = BasicBlock {
            statements: vec![
                Statement::Assign(Place::Local(LocalId(0)), Rvalue::Use(Operand::Constant(Constant::Int(20, Type::I32))), sp),
            ],
            terminator: Terminator::Goto(BasicBlockId(3)),
        };
        let bb3 = BasicBlock {
            statements: vec![],
            terminator: Terminator::Return(sp),
        };

        let body = MirBody { locals, blocks: vec![bb0, bb1, bb2, bb3], start_block: BasicBlockId(0) };
        let fid = FunctionId(0);
        let mut interp = Interpreter::new();
        interp.register_function(fid, "main", body);
        let result = interp.call_by_name("main", vec![]).unwrap();
        assert_eq!(result, Value::Int(10));
    }

    #[test]
    fn test_division_by_zero() {
        let sp = sp();
        let locals = vec![
            LocalDecl { id: LocalId(0), ty: Type::I32, span: sp, mutability: false, is_param: false },
        ];
        let block = BasicBlock {
            statements: vec![
                Statement::Assign(
                    Place::Local(LocalId(0)),
                    Rvalue::BinaryOp(
                        BinaryOp::Div,
                        Operand::Constant(Constant::Int(10, Type::I32)),
                        Operand::Constant(Constant::Int(0, Type::I32)),
                    ),
                    sp,
                ),
            ],
            terminator: Terminator::Return(sp),
        };
        let body = MirBody { locals, blocks: vec![block], start_block: BasicBlockId(0) };
        let fid = FunctionId(0);
        let mut interp = Interpreter::new();
        interp.register_function(fid, "main", body);
        let result = interp.call_by_name("main", vec![]);
        assert!(matches!(result, Err(InterpretError::DivisionByZero)));
    }

    #[test]
    fn test_aggregate() {
        // struct Point { x: i32, y: i32 } — aggregate with 2 fields
        let sp = sp();
        let locals = vec![
            LocalDecl { id: LocalId(0), ty: Type::I32, span: sp, mutability: false, is_param: false },
            LocalDecl { id: LocalId(1), ty: Type::I32, span: sp, mutability: false, is_param: false }, // point
            LocalDecl { id: LocalId(1000), ty: Type::I32, span: sp, mutability: false, is_param: false }, // temp field x
        ];
        let block = BasicBlock {
            statements: vec![
                Statement::StorageLive(LocalId(1), sp),
                Statement::Assign(
                    Place::Local(LocalId(1)),
                    Rvalue::Aggregate(vec![
                        Operand::Constant(Constant::Int(3, Type::I32)),
                        Operand::Constant(Constant::Int(4, Type::I32)),
                    ]),
                    sp,
                ),
                // read field 0 (x)
                Statement::StorageLive(LocalId(1000), sp),
                Statement::Assign(
                    Place::Local(LocalId(1000)),
                    Rvalue::Use(Operand::Copy(Place::Field {
                        base: Box::new(Place::Local(LocalId(1))),
                        field: FieldId(0),
                    })),
                    sp,
                ),
                Statement::Assign(Place::Local(LocalId(0)), Rvalue::Use(Operand::Copy(Place::Local(LocalId(1000)))), sp),
            ],
            terminator: Terminator::Return(sp),
        };
        let body = MirBody { locals, blocks: vec![block], start_block: BasicBlockId(0) };
        let fid = FunctionId(0);
        let mut interp = Interpreter::new();
        interp.register_function(fid, "main", body);
        let result = interp.call_by_name("main", vec![]).unwrap();
        assert_eq!(result, Value::Int(3));
    }
}
