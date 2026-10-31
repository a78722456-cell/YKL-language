use apl_ast::BinaryOp;
use apl_mir::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum VmError {
    #[error("Function '{0}' not found")]
    FunctionNotFound(String),
    #[error("Variable '{0:?}' not found in VM registers")]
    VariableNotFound(MirVar),
    #[error("Invalid operation on value")]
    InvalidOperation,
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Invalid memory access: {0}")]
    InvalidMemoryAccess(i64),
}

pub struct VmState {
    pub functions: Mutex<HashMap<String, MirFunction>>,
    pub heap: Mutex<HashMap<i64, Vec<MirValue>>>,
    pub next_alloc_id: AtomicI64,
}

#[derive(Clone)]
pub struct VirtualMachine {
    pub state: Arc<VmState>,
    pub output_buffer: Arc<Mutex<Vec<String>>>,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            state: Arc::new(VmState {
                functions: Mutex::new(HashMap::new()),
                heap: Mutex::new(HashMap::new()),
                next_alloc_id: AtomicI64::new(1),
            }),
            output_buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn register_function(&mut self, func: MirFunction) {
        self.state
            .functions
            .lock()
            .unwrap()
            .insert(func.name.clone(), func);
    }

    pub fn execute(&mut self, func_name: &str, args: Vec<MirValue>) -> Result<MirValue, VmError> {
        let func = {
            self.state
                .functions
                .lock()
                .unwrap()
                .get(func_name)
                .cloned()
                .ok_or_else(|| VmError::FunctionNotFound(func_name.to_string()))?
        };
        let mut registers: HashMap<MirVar, MirValue> = HashMap::new();

        for (i, param) in func.params.iter().enumerate() {
            if let Some(arg) = args.get(i) {
                registers.insert(*param, arg.clone());
            } else {
                registers.insert(*param, MirValue::ConstantInt(0));
            }
        }

        let mut current_block_id = MirBlockId(0);

        loop {
            let bb = func
                .blocks
                .iter()
                .find(|b| b.id == current_block_id)
                .ok_or(VmError::InvalidOperation)?;

            for inst in &bb.instructions {
                match inst {
                    MirInstruction::Assign(dest, val) => {
                        let eval_val = self.eval_value(val, &registers)?;
                        registers.insert(*dest, eval_val);
                    }
                    MirInstruction::Alloc(dest, _) => {
                        let id = self.state.next_alloc_id.fetch_add(1, Ordering::SeqCst);
                        self.state
                            .heap
                            .lock()
                            .unwrap()
                            .insert(id, vec![MirValue::ConstantInt(0)]);
                        registers.insert(*dest, MirValue::ConstantInt(id));
                    }

                    MirInstruction::Binary(dest, op, left, right) => {
                        let l_val = self.eval_value(left, &registers)?;
                        let r_val = self.eval_value(right, &registers)?;
                        let res = self.eval_binary(*op, l_val, r_val)?;
                        registers.insert(*dest, res);
                    }
                    MirInstruction::Unary(dest, _, operand) => {
                        let o_val = self.eval_value(operand, &registers)?;
                        registers.insert(*dest, o_val);
                    }
                    MirInstruction::StringConcat(dest, left, right) => {
                        let l_val = self.eval_value(left, &registers)?;
                        let r_val = self.eval_value(right, &registers)?;
                        if let (MirValue::ConstantString(l), MirValue::ConstantString(r)) =
                            (l_val, r_val)
                        {
                            let mut res = l;
                            res.push_str(&r);
                            registers.insert(*dest, MirValue::ConstantString(res));
                        } else {
                            return Err(VmError::InvalidOperation);
                        }
                    }
                    MirInstruction::StructAlloc(dest, _, fields) => {
                        let mut vals = Vec::new();
                        for f in fields {
                            vals.push(self.eval_value(f, &registers)?);
                        }
                        let id = self.state.next_alloc_id.fetch_add(1, Ordering::SeqCst);
                        self.state.heap.lock().unwrap().insert(id, vals);
                        registers.insert(*dest, MirValue::ConstantInt(id));
                    }
                    MirInstruction::ArrayAlloc(dest, elems) => {
                        let mut vals = Vec::new();
                        for e in elems {
                            vals.push(self.eval_value(e, &registers)?);
                        }
                        let id = self.state.next_alloc_id.fetch_add(1, Ordering::SeqCst);
                        self.state.heap.lock().unwrap().insert(id, vals);
                        registers.insert(*dest, MirValue::ConstantInt(id));
                    }
                    MirInstruction::ArrayAllocDynamic(dest, size_val) => {
                        let size = match self.eval_value(size_val, &registers)? {
                            MirValue::ConstantInt(v) => v as usize,
                            _ => return Err(VmError::InvalidMemoryAccess(0)),
                        };
                        let vals = vec![MirValue::ConstantInt(0); size]; // Zero init
                        let id = self.state.next_alloc_id.fetch_add(1, Ordering::SeqCst);
                        self.state.heap.lock().unwrap().insert(id, vals);
                        registers.insert(*dest, MirValue::ConstantInt(id));
                    }
                    MirInstruction::FieldGet(dest, base, field) => {
                        let base_ptr = match self.eval_value(base, &registers)? {
                            MirValue::ConstantInt(p) => p,
                            _ => return Err(VmError::InvalidMemoryAccess(0)),
                        };
                        let idx = field.parse::<usize>().unwrap();
                        let heap = self.state.heap.lock().unwrap();
                        let obj = heap
                            .get(&base_ptr)
                            .ok_or(VmError::InvalidMemoryAccess(base_ptr))?;
                        registers.insert(*dest, obj[idx].clone());
                    }
                    MirInstruction::FieldSet(base, field, val) => {
                        let base_ptr = match self.eval_value(base, &registers)? {
                            MirValue::ConstantInt(p) => p,
                            _ => return Err(VmError::InvalidMemoryAccess(0)),
                        };
                        let idx = field.parse::<usize>().unwrap();
                        let v = self.eval_value(val, &registers)?;
                        let mut heap = self.state.heap.lock().unwrap();
                        let obj = heap
                            .get_mut(&base_ptr)
                            .ok_or(VmError::InvalidMemoryAccess(base_ptr))?;
                        obj[idx] = v;
                    }
                    MirInstruction::ArrayGet(dest, base, idx) => {
                        let base_ptr = match self.eval_value(base, &registers)? {
                            MirValue::ConstantInt(p) => p,
                            _ => return Err(VmError::InvalidMemoryAccess(0)),
                        };
                        let i = match self.eval_value(idx, &registers)? {
                            MirValue::ConstantInt(v) => v as usize,
                            _ => return Err(VmError::InvalidMemoryAccess(0)),
                        };
                        let heap = self.state.heap.lock().unwrap();
                        let obj = heap
                            .get(&base_ptr)
                            .ok_or(VmError::InvalidMemoryAccess(base_ptr))?;
                        if i >= obj.len() {
                            return Err(VmError::InvalidMemoryAccess(base_ptr));
                        }
                        registers.insert(*dest, obj[i].clone());
                    }
                    MirInstruction::ArraySet(base, idx, val) => {
                        let base_ptr = match self.eval_value(base, &registers)? {
                            MirValue::ConstantInt(p) => p,
                            _ => return Err(VmError::InvalidMemoryAccess(0)),
                        };
                        let i = match self.eval_value(idx, &registers)? {
                            MirValue::ConstantInt(v) => v as usize,
                            _ => return Err(VmError::InvalidMemoryAccess(0)),
                        };
                        let v = self.eval_value(val, &registers)?;
                        let mut heap = self.state.heap.lock().unwrap();
                        let obj = heap
                            .get_mut(&base_ptr)
                            .ok_or(VmError::InvalidMemoryAccess(base_ptr))?;
                        if i >= obj.len() {
                            return Err(VmError::InvalidMemoryAccess(base_ptr));
                        }
                        obj[i] = v;
                    }
                    MirInstruction::Drop(var) => {
                        let ptr = match self.eval_value(&MirValue::Var(*var), &registers) {
                            Ok(MirValue::ConstantInt(p)) => p,
                            _ => 0,
                        };
                        if ptr != 0 {
                            self.state.heap.lock().unwrap().remove(&ptr);
                        }
                    }
                    MirInstruction::ExternCall(dest, callee, call_args) => {
                        let mut eval_args = Vec::new();
                        for a in call_args {
                            eval_args.push(self.eval_value(a, &registers)?);
                        }
                        // Built-in syscall mock for MVP File I/O & Network
                        let res = match callee.as_str() {
                            // Temporary mocks
                            "open" => MirValue::ConstantInt(3), // fake fd
                            "read" => MirValue::ConstantInt(0), // EOF
                            "close" => MirValue::ConstantInt(0), // Success
                            _ => {
                                println!("WARNING: ExternCall to {} not simulated in VM, returning 0", callee);
                                MirValue::ConstantInt(0)
                            }
                        };
                        registers.insert(*dest, res);
                    }
                    MirInstruction::Call(dest, callee, call_args) => {
                        let mut eval_args = Vec::new();
                        for a in call_args {
                            eval_args.push(self.eval_value(a, &registers)?);
                        }
                        if callee == "println" {
                            let msg = match eval_args.get(0) {
                                Some(MirValue::ConstantString(s)) => s.clone(),
                                Some(MirValue::ConstantInt(i)) => i.to_string(),
                                Some(MirValue::ConstantFloat(f)) => f.to_string(),
                                Some(MirValue::ConstantBool(b)) => b.to_string(),
                                _ => "".to_string(),
                            };
                            println!("{}", msg);
                            self.output_buffer.lock().unwrap().push(msg);
                            registers.insert(*dest, MirValue::ConstantInt(0));
                        } else if callee == "spawn" {
                            if let Some(MirValue::ConstantString(thread_func)) =
                                eval_args.get(0).cloned()
                            {
                                let mut thread_vm = self.clone();
                                let thread_args: Vec<MirValue> =
                                    eval_args.into_iter().skip(1).collect();

                                std::thread::spawn(move || {
                                    if let Err(e) = thread_vm.execute(&thread_func, thread_args) {
                                        println!("Thread error: {:?}", e);
                                    }
                                });
                                registers.insert(*dest, MirValue::ConstantInt(0)); // Should return a thread handle, returning 0 for now
                            } else {
                                return Err(VmError::InvalidOperation);
                            }
                        } else if callee == "sleep" {
                            if let Some(MirValue::ConstantInt(ms)) = eval_args.get(0) {
                                std::thread::sleep(std::time::Duration::from_millis(*ms as u64));
                            }
                            registers.insert(*dest, MirValue::ConstantInt(0));
                        } else {
                            let res = self.execute(callee, eval_args)?;
                            registers.insert(*dest, res);
                        }
                    }
                }
            }

            match &bb.terminator {
                MirTerminator::Return(opt_val) => {
                    if let Some(val) = opt_val {
                        return self.eval_value(val, &registers);
                    } else {
                        return Ok(MirValue::ConstantInt(0));
                    }
                }
                MirTerminator::Branch(target) => {
                    current_block_id = *target;
                }
                MirTerminator::CondBranch(cond, then_b, else_b) => {
                    let cond_val = self.eval_value(cond, &registers)?;
                    let is_true = match cond_val {
                        MirValue::ConstantBool(b) => b,
                        MirValue::ConstantInt(i) => i != 0,
                        _ => false,
                    };
                    if is_true {
                        current_block_id = *then_b;
                    } else {
                        current_block_id = *else_b;
                    }
                }
                MirTerminator::Unreachable => {
                    return Err(VmError::InvalidOperation);
                }
            }
        }
    }

    fn eval_value(
        &self,
        val: &MirValue,
        registers: &HashMap<MirVar, MirValue>,
    ) -> Result<MirValue, VmError> {
        match val {
            MirValue::Var(v) => registers
                .get(v)
                .cloned()
                .ok_or(VmError::VariableNotFound(*v)),
            other => Ok(other.clone()),
        }
    }

    fn eval_binary(
        &self,
        op: BinaryOp,
        left: MirValue,
        right: MirValue,
    ) -> Result<MirValue, VmError> {
        match (left, right) {
            (MirValue::ConstantInt(l), MirValue::ConstantInt(r)) => match op {
                BinaryOp::Add => Ok(MirValue::ConstantInt(l.wrapping_add(r))),
                BinaryOp::Sub => Ok(MirValue::ConstantInt(l.wrapping_sub(r))),
                BinaryOp::Mul => Ok(MirValue::ConstantInt(l.wrapping_mul(r))),
                BinaryOp::Div => {
                    if r == 0 {
                        Err(VmError::DivisionByZero)
                    } else {
                        Ok(MirValue::ConstantInt(l.wrapping_div(r)))
                    }
                }
                BinaryOp::Eq => Ok(MirValue::ConstantBool(l == r)),
                BinaryOp::Neq => Ok(MirValue::ConstantBool(l != r)),
                BinaryOp::Lt => Ok(MirValue::ConstantBool(l < r)),
                BinaryOp::Lte => Ok(MirValue::ConstantBool(l <= r)),
                BinaryOp::Gt => Ok(MirValue::ConstantBool(l > r)),
                BinaryOp::Gte => Ok(MirValue::ConstantBool(l >= r)),
                _ => Err(VmError::InvalidOperation),
            },
            (MirValue::ConstantString(l), MirValue::ConstantString(r)) => match op {
                BinaryOp::Add => {
                    let mut res = l.clone();
                    res.push_str(&r);
                    Ok(MirValue::ConstantString(res))
                }
                BinaryOp::Eq => Ok(MirValue::ConstantBool(l == r)),
                BinaryOp::Neq => Ok(MirValue::ConstantBool(l != r)),
                _ => Err(VmError::InvalidOperation),
            },
            _ => Err(VmError::InvalidOperation),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apl_ast::*;
    use apl_mir::MirBuilder;

    #[test]
    fn test_vm_execution() {
        let func = Function {
            name: "factorial_step".to_string(),
            generics: vec![],
            params: vec![Param {
                name: "n".to_string(),
                ty: Type::I32,
                span: Span::dummy(),
            }],
            return_type: Type::I32,
            is_async: false,
            is_pub: true,
            body: Expr {
                kind: ExprKind::Binary(
                    BinaryOp::Mul,
                    Box::new(Expr {
                        kind: ExprKind::Ident("n".to_string()),
                        span: Span::dummy(),
                    }),
                    Box::new(Expr {
                        kind: ExprKind::Literal(Literal::Int(5)),
                        span: Span::dummy(),
                    }),
                ),
                span: Span::dummy(),
            },
            span: Span::dummy(),
        };

        use std::collections::HashMap;
        let structs = HashMap::new();
        let enums = HashMap::new();
        let externs = HashMap::new();
        let type_map = HashMap::new();
        let mut builder = MirBuilder::new(&structs, &enums, &externs, &type_map);
        let mir = builder.lower_function(&func);

        let mut vm = VirtualMachine::new();
        vm.register_function(mir);
        let res = vm
            .execute("factorial_step", vec![MirValue::ConstantInt(6)])
            .unwrap();
        assert_eq!(res, MirValue::ConstantInt(30));
    }
}
