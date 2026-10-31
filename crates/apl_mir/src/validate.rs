use crate::*;
use std::collections::HashSet;

impl MirBody {
    pub fn validate(&self) -> Result<(), String> {
        let max_block = self.blocks.len() as u32;
        if self.start_block.0 >= max_block {
            return Err(format!("Start block bb{} is out of bounds", self.start_block.0));
        }

        let mut local_ids = HashSet::new();
        for local in &self.locals {
            local_ids.insert(local.id);
        }

        for (i, block) in self.blocks.iter().enumerate() {
            let bb = BasicBlockId(i as u32);
            for stmt in &block.statements {
                match stmt {
                    Statement::Assign(p, r, _) => {
                        self.validate_place(p, &local_ids)?;
                        self.validate_rvalue(r, &local_ids)?;
                    }
                    Statement::StorageLive(id, _) | Statement::StorageDead(id, _) => {
                        if !local_ids.contains(id) {
                            return Err(format!("Storage instruction for unknown LocalId: {:?}", id));
                        }
                    }
                    Statement::Drop(p, _) => {
                        self.validate_place(p, &local_ids)?;
                    }
                    Statement::Nop => {}
                }
            }

            match &block.terminator {
                Terminator::Goto(target) => {
                    if target.0 >= max_block {
                        return Err(format!("bb{}: Goto target bb{} out of bounds", bb.0, target.0));
                    }
                }
                Terminator::SwitchInt { targets, otherwise, .. } => {
                    for (_, target) in targets {
                        if target.0 >= max_block {
                            return Err(format!("bb{}: SwitchInt target bb{} out of bounds", bb.0, target.0));
                        }
                    }
                    if otherwise.0 >= max_block {
                        return Err(format!("bb{}: SwitchInt otherwise bb{} out of bounds", bb.0, otherwise.0));
                    }
                }
                Terminator::Call { target, destination, .. } => {
                    if target.0 >= max_block {
                        return Err(format!("bb{}: Call target bb{} out of bounds", bb.0, target.0));
                    }
                    self.validate_place(destination, &local_ids)?;
                }
                Terminator::Return(_) | Terminator::Unreachable | Terminator::Resume => {}
                Terminator::Drop { place, target, .. } => {
                    if target.0 >= max_block {
                        return Err(format!("bb{}: Drop target bb{} out of bounds", bb.0, target.0));
                    }
                    self.validate_place(place, &local_ids)?;
                }
            }
        }
        Ok(())
    }

    fn validate_place(&self, place: &Place, local_ids: &HashSet<LocalId>) -> Result<(), String> {
        match place {
            Place::Local(id) => {
                if !local_ids.contains(id) {
                    return Err(format!("Place references unknown local {:?}", id));
                }
            }
            Place::Field { base, .. } => self.validate_place(base, local_ids)?,
            Place::Index { base, index } => {
                self.validate_place(base, local_ids)?;
                if !local_ids.contains(index) {
                    return Err(format!("Index references unknown local {:?}", index));
                }
            }
            Place::Deref { base } => self.validate_place(base, local_ids)?,
        }
        Ok(())
    }

    fn validate_rvalue(&self, rvalue: &Rvalue, local_ids: &HashSet<LocalId>) -> Result<(), String> {
        match rvalue {
            Rvalue::Use(op) => self.validate_operand(op, local_ids)?,
            Rvalue::Ref(_, _, place) => self.validate_place(place, local_ids)?,
            Rvalue::BinaryOp(_, l, r) => {
                self.validate_operand(l, local_ids)?;
                self.validate_operand(r, local_ids)?;
            }
            Rvalue::UnaryOp(_, op) => self.validate_operand(op, local_ids)?,
            Rvalue::Aggregate(ops) => {
                for op in ops {
                    self.validate_operand(op, local_ids)?;
                }
            },
        }
        Ok(())
    }

    fn validate_operand(&self, op: &Operand, local_ids: &HashSet<LocalId>) -> Result<(), String> {
        match op {
            Operand::Copy(place) | Operand::Move(place) => self.validate_place(place, local_ids),
            Operand::Constant(_) => Ok(()),
        }
    }
}
