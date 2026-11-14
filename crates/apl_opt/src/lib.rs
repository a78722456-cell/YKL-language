use apl_ast::BinaryOp;
use apl_mir::{Constant, LocalId, MirBody, Operand, Place, Rvalue, Statement};
use std::collections::HashMap;

pub struct MirOptimizer;

impl MirOptimizer {
    /// Выполняет оптимизацию графа MIR:
    /// 1) Свёртка константных выражений (Constant Folding, например `1000 * 20` -> `20000`)
    /// 2) Удаление пустых инструкций `Statement::Nop`
    pub fn optimize_body(body: &mut MirBody) {
        for block in &mut body.blocks {
            let mut const_map: HashMap<LocalId, Constant> = HashMap::new();
            for stmt in &mut block.statements {
                if let Statement::Assign(place, rvalue, _span) = stmt {
                    // Try to fold rvalue using known constants
                    if let Some(folded) = Self::try_fold_rvalue(rvalue, &const_map) {
                        *rvalue = folded;
                    }
                    // Record if target is a local holding a constant
                    if let Place::Local(id) = place {
                        if let Rvalue::Use(Operand::Constant(c)) = rvalue {
                            const_map.insert(*id, c.clone());
                        } else {
                            const_map.remove(id);
                        }
                    }
                }
            }
            block.statements.retain(|s| !matches!(s, Statement::Nop));
        }
    }

    fn resolve_const<'a>(op: &'a Operand, const_map: &'a HashMap<LocalId, Constant>) -> Option<&'a Constant> {
        match op {
            Operand::Constant(c) => Some(c),
            Operand::Copy(Place::Local(id)) | Operand::Move(Place::Local(id)) => const_map.get(id),
            _ => None,
        }
    }

    fn try_fold_rvalue(rv: &Rvalue, const_map: &HashMap<LocalId, Constant>) -> Option<Rvalue> {
        match rv {
            Rvalue::BinaryOp(op, l, r) => {
                let l_c = Self::resolve_const(l, const_map)?;
                let r_c = Self::resolve_const(r, const_map)?;

                match (l_c, r_c) {
                    (Constant::Int(a, ty), Constant::Int(b, _)) => {
                        let folded_const = match op {
                            BinaryOp::Add => Constant::Int(a.wrapping_add(*b), ty.clone()),
                            BinaryOp::Sub => Constant::Int(a.wrapping_sub(*b), ty.clone()),
                            BinaryOp::Mul => Constant::Int(a.wrapping_mul(*b), ty.clone()),
                            BinaryOp::Div if *b != 0 => Constant::Int(a / b, ty.clone()),
                            BinaryOp::Mod if *b != 0 => Constant::Int(a % b, ty.clone()),
                            BinaryOp::BitAnd => Constant::Int(a & b, ty.clone()),
                            BinaryOp::BitOr => Constant::Int(a | b, ty.clone()),
                            BinaryOp::BitXor => Constant::Int(a ^ b, ty.clone()),
                            BinaryOp::Shl => Constant::Int(a.wrapping_shl(*b as u32), ty.clone()),
                            BinaryOp::Shr => Constant::Int(a.wrapping_shr(*b as u32), ty.clone()),
                            BinaryOp::Eq => Constant::Bool(a == b),
                            BinaryOp::Neq => Constant::Bool(a != b),
                            BinaryOp::Lt => Constant::Bool(a < b),
                            BinaryOp::Lte => Constant::Bool(a <= b),
                            BinaryOp::Gt => Constant::Bool(a > b),
                            BinaryOp::Gte => Constant::Bool(a >= b),
                            _ => return None,
                        };
                        Some(Rvalue::Use(Operand::Constant(folded_const)))
                    }
                    (Constant::Bool(a), Constant::Bool(b)) => {
                        let folded_const = match op {
                            BinaryOp::And => Constant::Bool(*a && *b),
                            BinaryOp::Or => Constant::Bool(*a || *b),
                            BinaryOp::Eq => Constant::Bool(a == b),
                            BinaryOp::Neq => Constant::Bool(a != b),
                            _ => return None,
                        };
                        Some(Rvalue::Use(Operand::Constant(folded_const)))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apl_ast::{Span, Type};
    use apl_mir::{BasicBlock, BasicBlockId, Terminator};

    #[test]
    fn test_constant_folding() {
        let mut body = MirBody {
            locals: vec![],
            blocks: vec![BasicBlock {
                statements: vec![
                    Statement::Assign(
                        Place::Local(LocalId(1)),
                        Rvalue::BinaryOp(
                            BinaryOp::Mul,
                            Operand::Constant(Constant::Int(1000, Type::I32)),
                            Operand::Constant(Constant::Int(20, Type::I32)),
                        ),
                        Span::dummy(),
                    ),
                    Statement::Nop,
                ],
                terminator: Terminator::Return(Span::dummy()),
            }],
            start_block: BasicBlockId(0),
        };

        MirOptimizer::optimize_body(&mut body);

        assert_eq!(body.blocks[0].statements.len(), 1);
        if let Statement::Assign(_, Rvalue::Use(Operand::Constant(Constant::Int(val, _))), _) =
            &body.blocks[0].statements[0]
        {
            assert_eq!(*val, 20000);
        } else {
            panic!("Expected folded constant");
        }
    }
}
