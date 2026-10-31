use apl_ast::BinaryOp;
use apl_mir::*;
use std::collections::{HashMap, HashSet};

pub struct MirOptimizer;

impl MirOptimizer {
    fn mark_used(val: &MirValue, used_vars: &mut HashSet<MirVar>) {
        if let MirValue::Var(v) = val {
            used_vars.insert(*v);
        }
    }

    pub fn optimize_function(func: &mut MirFunction) {
        let mut changed = true;
        while changed {
            changed = false;
            changed |= Self::constant_folding(func);
            changed |= Self::dead_code_elimination(func);
        }
    }

    fn constant_folding(func: &mut MirFunction) -> bool {
        let mut changed = false;

        for bb in &mut func.blocks {
            let mut const_map: HashMap<MirVar, MirValue> = HashMap::new();
            for inst in &mut bb.instructions {
                // Replace operands if they are known constants
                match inst {
                    MirInstruction::Assign(_, val) => {
                        if let MirValue::Var(v) = val {
                            if let Some(c) = const_map.get(v) {
                                *val = c.clone();
                                changed = true;
                            }
                        }
                    }
                    MirInstruction::Binary(_, _, left, right) => {
                        if let MirValue::Var(v) = left {
                            if let Some(c) = const_map.get(v) {
                                *left = c.clone();
                                changed = true;
                            }
                        }
                        if let MirValue::Var(v) = right {
                            if let Some(c) = const_map.get(v) {
                                *right = c.clone();
                                changed = true;
                            }
                        }
                    }
                    _ => {}
                }

                // Fold binary constants
                let folded_info = if let MirInstruction::Binary(dest, op, left, right) = inst {
                    if let (MirValue::ConstantInt(l), MirValue::ConstantInt(r)) = (left, right) {
                        match op {
                            BinaryOp::Add => Some((*dest, l.wrapping_add(*r))),
                            BinaryOp::Sub => Some((*dest, l.wrapping_sub(*r))),
                            BinaryOp::Mul => Some((*dest, l.wrapping_mul(*r))),
                            BinaryOp::Div if *r != 0 => Some((*dest, l.wrapping_div(*r))),
                            _ => None,
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some((dest, f)) = folded_info {
                    *inst = MirInstruction::Assign(dest, MirValue::ConstantInt(f));
                    const_map.insert(dest, MirValue::ConstantInt(f));
                    changed = true;
                    continue;
                }

                // Record constant assignments
                if let MirInstruction::Assign(dest, val) = inst {
                    match val {
                        MirValue::ConstantInt(_)
                        | MirValue::ConstantFloat(_)
                        | MirValue::ConstantBool(_) => {
                            const_map.insert(*dest, val.clone());
                        }
                        _ => {}
                    }
                }
            }
        }
        changed
    }

    fn dead_code_elimination(func: &mut MirFunction) -> bool {
        let mut used_vars = HashSet::new();

        // Collect all used variables
        for bb in &func.blocks {
            for inst in &bb.instructions {
                match inst {
                    MirInstruction::Assign(_, val) => {
                        if let MirValue::Var(v) = val {
                            used_vars.insert(*v);
                        }
                    }
                    MirInstruction::Binary(_, _, left, right)
                    | MirInstruction::StringConcat(_, left, right) => {
                        Self::mark_used(left, &mut used_vars);
                        Self::mark_used(right, &mut used_vars);
                    }
                    MirInstruction::Unary(_, _, op) => {
                        Self::mark_used(op, &mut used_vars);
                    }
                    MirInstruction::Call(_, _, args) | MirInstruction::ExternCall(_, _, args) => {
                        for a in args {
                            Self::mark_used(a, &mut used_vars);
                        }
                    }
                    MirInstruction::StructAlloc(_, _, fields)
                    | MirInstruction::ArrayAlloc(_, fields) => {
                        for f in fields {
                            Self::mark_used(f, &mut used_vars);
                        }
                    }
                    MirInstruction::FieldGet(_, base, _) => {
                        Self::mark_used(base, &mut used_vars);
                    }
                    MirInstruction::ArrayGet(_, base, idx) => {
                        Self::mark_used(base, &mut used_vars);
                        Self::mark_used(idx, &mut used_vars);
                    }
                    MirInstruction::FieldSet(base, _, val) => {
                        Self::mark_used(base, &mut used_vars);
                        Self::mark_used(val, &mut used_vars);
                    }
                    MirInstruction::ArraySet(base, idx, val) => {
                        Self::mark_used(base, &mut used_vars);
                        Self::mark_used(idx, &mut used_vars);
                        Self::mark_used(val, &mut used_vars);
                    }
                    MirInstruction::Drop(_) => {}
                    MirInstruction::ArrayAllocDynamic(_, size_val) => {
                        Self::mark_used(size_val, &mut used_vars);
                    }
                    _ => {}
                }
            }
            match &bb.terminator {
                MirTerminator::Return(Some(MirValue::Var(v))) => {
                    used_vars.insert(*v);
                }
                MirTerminator::CondBranch(MirValue::Var(v), _, _) => {
                    used_vars.insert(*v);
                }
                _ => {}
            }
        }

        let mut changed = false;
        for bb in &mut func.blocks {
            let orig_len = bb.instructions.len();
            bb.instructions.retain(|inst| match inst {
                MirInstruction::Assign(dest, _)
                | MirInstruction::Binary(dest, _, _, _)
                | MirInstruction::Unary(dest, _, _)
                | MirInstruction::FieldGet(dest, _, _)
                | MirInstruction::ArrayGet(dest, _, _)
                | MirInstruction::StringConcat(dest, _, _) => used_vars.contains(dest),
                MirInstruction::Call(_, _, _)
                | MirInstruction::ExternCall(_, _, _)
                | MirInstruction::Alloc(_, _)
                | MirInstruction::StructAlloc(_, _, _)
                | MirInstruction::ArrayAlloc(_, _)
                | MirInstruction::ArrayAllocDynamic(_, _)
                | MirInstruction::FieldSet(_, _, _)
                | MirInstruction::ArraySet(_, _, _)
                | MirInstruction::Drop(_) => true,
            });
            if bb.instructions.len() != orig_len {
                changed = true;
            }
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apl_ast::BinaryOp;

    #[test]
    fn test_constant_folding_and_dce() {
        let mut bb0 = MirBasicBlock::new(MirBlockId(0));
        // v0 = 10
        bb0.instructions
            .push(MirInstruction::Assign(MirVar(0), MirValue::ConstantInt(10)));
        // v1 = 20
        bb0.instructions
            .push(MirInstruction::Assign(MirVar(1), MirValue::ConstantInt(20)));
        // v2 = v0 + v1 -> should fold to v2 = 30
        bb0.instructions.push(MirInstruction::Binary(
            MirVar(2),
            BinaryOp::Add,
            MirValue::Var(MirVar(0)),
            MirValue::Var(MirVar(1)),
        ));
        // v3 = 999 (unused) -> should be eliminated by DCE
        bb0.instructions.push(MirInstruction::Assign(
            MirVar(3),
            MirValue::ConstantInt(999),
        ));
        bb0.terminator = MirTerminator::Return(Some(MirValue::Var(MirVar(2))));

        let mut func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            blocks: vec![bb0],
        };

        MirOptimizer::optimize_function(&mut func);

        // Check instructions
        let insts = &func.blocks[0].instructions;
        // v3 should be removed
        assert!(
            !insts
                .iter()
                .any(|i| matches!(i, MirInstruction::Assign(MirVar(3), _)))
        );
        // v2 should be folded to 30
        assert!(insts.iter().any(|i| matches!(
            i,
            MirInstruction::Assign(MirVar(2), MirValue::ConstantInt(30))
        )));
    }
}
