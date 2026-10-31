import re

with open('crates/apl_mir/src/validate.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# Let's restore the match statement for Statement in validate.rs
start = text.find('match stmt {')
end = text.find('}', text.find('Statement::Nop => {}')) + 1

replacement = '''match stmt {
                Statement::Assign(p, r, _) => {
                    self.validate_place(p, local_ids)?;
                    self.validate_rvalue(r, local_ids, func_ids)?;
                }
                Statement::StorageLive(id, _) | Statement::StorageDead(id, _) => {
                    if !local_ids.contains(id) {
                        return Err(format!("Storage instruction for unknown LocalId: {:?}", id));
                    }
                }
                Statement::Drop(p, _) => {
                    self.validate_place(p, local_ids)?;
                }
                Statement::Nop => {}
            }'''

text = text[:start] + replacement + text[end:]

with open('crates/apl_mir/src/validate.rs', 'w', encoding='utf-8') as f:
    f.write(text)

# Also fix analysis.rs LivenessAnalysis
with open('crates/apl_mir/src/analysis.rs', 'r', encoding='utf-8') as f:
    text2 = f.read()

start2 = text2.find('fn transfer_statement(&self, stmt: &crate::Statement, state: &mut Self::Domain) {')
end2 = text2.find('}', text2.find('Statement::Nop => {}')) + 1

replacement2 = '''fn transfer_statement(&self, stmt: &crate::Statement, state: &mut Self::Domain) {
        match stmt {
            crate::Statement::Assign(crate::Place::Local(id), rvalue, _) => {
                state.remove(id);
                // Also usages in rvalue make things live
                match rvalue {
                    crate::Rvalue::Use(op) | crate::Rvalue::UnaryOp(_, op) => {
                        if let crate::Operand::Copy(crate::Place::Local(uid)) | crate::Operand::Move(crate::Place::Local(uid)) = op {
                            state.insert(*uid);
                        }
                    }
                    crate::Rvalue::BinaryOp(_, l, r) => {
                        if let crate::Operand::Copy(crate::Place::Local(uid)) | crate::Operand::Move(crate::Place::Local(uid)) = l {
                            state.insert(*uid);
                        }
                        if let crate::Operand::Copy(crate::Place::Local(uid)) | crate::Operand::Move(crate::Place::Local(uid)) = r {
                            state.insert(*uid);
                        }
                    }
                    crate::Rvalue::Ref(_, _, crate::Place::Local(uid)) => {
                        state.insert(*uid);
                    }
                    crate::Rvalue::Aggregate(ops) => {
                        for op in ops {
                            if let crate::Operand::Copy(crate::Place::Local(uid)) | crate::Operand::Move(crate::Place::Local(uid)) = op {
                                state.insert(*uid);
                            }
                        }
                    }
                    _ => {}
                }
            }
            crate::Statement::StorageDead(id, _) => {}
            crate::Statement::StorageLive(id, _) => {}
            crate::Statement::Drop(crate::Place::Local(id), _) => {
                state.insert(*id); // Drop is a use!
            }
            _ => {}
        }
    }'''

text2 = text2[:start2] + replacement2 + text2[end2:]
with open('crates/apl_mir/src/analysis.rs', 'w', encoding='utf-8') as f:
    f.write(text2)

# Also fix apl_borrowck check_moves
with open('crates/apl_borrowck/src/lib.rs', 'r', encoding='utf-8') as f:
    text3 = f.read()

start3 = text3.find('match stmt {')
end3 = text3.find('}', text3.find('_ => {}')) + 1

replacement3 = '''match stmt {
                Statement::Assign(place, rvalue, span) => {
                    check_rvalue_uses(rvalue, &current_uninit, &current_ever, *span, errors);
                    if let Place::Local(id) = place {
                        current_uninit.remove(id);
                        current_ever.insert(*id);
                    }
                    if let Rvalue::Use(Operand::Move(Place::Local(id))) = rvalue {
                        current_uninit.insert(*id);
                    }
                }
                Statement::StorageDead(id, _) => {
                    current_uninit.insert(*id);
                }
                Statement::Drop(Place::Local(id), _) => {
                    current_uninit.insert(*id);
                }
                _ => {}
            }'''

text3 = text3[:start3] + replacement3 + text3[end3:]
with open('crates/apl_borrowck/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(text3)

print("done")
