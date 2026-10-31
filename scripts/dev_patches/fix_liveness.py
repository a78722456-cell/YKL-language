import os

with open('crates/apl_mir/src/analysis.rs', 'r', encoding='utf-8') as f:
    text = f.read()

liveness_code = '''
#[derive(Clone, PartialEq)]
pub struct LivenessAnalysis;

impl DataflowAnalysis for LivenessAnalysis {
    type Domain = HashSet<LocalId>;

    fn is_forward(&self) -> bool { false }

    fn entry_state(&self, _body: &MirBody) -> Self::Domain {
        HashSet::new()
    }

    fn bottom_value(&self, _body: &MirBody) -> Self::Domain {
        HashSet::new()
    }

    fn join(&self, state1: &mut Self::Domain, state2: &Self::Domain) -> bool {
        let mut changed = false;
        for item in state2 {
            if state1.insert(*item) {
                changed = true;
            }
        }
        changed
    }

    fn transfer_statement(&self, stmt: &Statement, state: &mut Self::Domain) {
        match stmt {
            Statement::Assign(place, rvalue, _) => {
                if let Place::Local(id) = place {
                    state.remove(id); // Written to, no longer live before this
                }
                
                // Used variables become live
                let mut add_uses = |op: &Operand| {
                    match op {
                        Operand::Copy(Place::Local(id)) | Operand::Move(Place::Local(id)) => {
                            state.insert(*id);
                        }
                        _ => {}
                    }
                };

                match rvalue {
                    Rvalue::Use(op) | Rvalue::UnaryOp(_, op) => add_uses(op),
                    Rvalue::BinaryOp(_, l, r) => {
                        add_uses(l);
                        add_uses(r);
                    }
                    Rvalue::Ref(_, _, Place::Local(id)) => {
                        state.insert(*id);
                    }
                    _ => {}
                }
            }
            Statement::StorageDead(id, _) | Statement::StorageLive(id, _) => {
                // If it is dead or just born, it is not live before this point
                state.remove(id);
            }
            _ => {}
        }
    }

    fn transfer_terminator(&self, term: &Terminator, state: &mut Self::Domain) {
        match term {
            Terminator::SwitchInt { discriminant, .. } => {
                if let Operand::Copy(Place::Local(id)) | Operand::Move(Place::Local(id)) = discriminant {
                    state.insert(*id);
                }
            }
            Terminator::Return(_) => {
                // Return value is always used (if it's not void)
                // Assuming _0 is the return value, it is live at exit!
                state.insert(LocalId(0));
            }
            Terminator::Call { args, destination: Place::Local(dest_id), .. } => {
                state.remove(dest_id);
                for arg in args {
                    if let Operand::Copy(Place::Local(id)) | Operand::Move(Place::Local(id)) = arg {
                        state.insert(*id);
                    }
                }
            }
            Terminator::Drop { place: Place::Local(id), .. } => {
                state.insert(*id);
            }
            _ => {}
        }
    }
}
'''

text += '\n' + liveness_code

with open('crates/apl_mir/src/analysis.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print("done")
