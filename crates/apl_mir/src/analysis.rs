use crate::*;
use std::collections::HashSet;
use crate::dataflow::{DataflowAnalysis, DataflowResults, run_analysis};

#[derive(Clone, PartialEq)]
pub struct MaybeUninitializedLocals {
    pub uninit: HashSet<LocalId>,
}

impl DataflowAnalysis for MaybeUninitializedLocals {
    type Domain = HashSet<LocalId>;

    fn is_forward(&self) -> bool { true }

    fn entry_state(&self, body: &MirBody) -> Self::Domain {
        let mut state = HashSet::new();
        for local in &body.locals {
            if !local.is_param {
                state.insert(local.id);
            }
        }
        state
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
                match rvalue {
                    Rvalue::Use(Operand::Move(Place::Local(id))) => { state.insert(*id); }
                    Rvalue::UnaryOp(_, Operand::Move(Place::Local(id))) => { state.insert(*id); }
                    Rvalue::BinaryOp(_, Operand::Move(Place::Local(l_id)), Operand::Move(Place::Local(r_id))) => {
                        state.insert(*l_id);
                        state.insert(*r_id);
                    }
                    Rvalue::Aggregate(ops) => {
                        for op in ops {
                            if let Operand::Move(Place::Local(id)) = op {
                                state.insert(*id);
                            }
                        }
                    }
                    _ => {}
                }
                if let Place::Local(id) = place {
                    state.remove(id);
                }
            }
            Statement::StorageDead(id, _) => {
                state.insert(*id);
            }
            Statement::Drop(Place::Local(id), _) => {
                state.insert(*id);
            }
            _ => {}
        }
    }

    fn transfer_terminator(&self, term: &Terminator, state: &mut Self::Domain) {
        match term {
            Terminator::Drop { place: Place::Local(id), .. } => {
                state.insert(*id);
            }
            Terminator::Call { args, destination: Place::Local(dest_id), .. } => {
                for arg in args {
                    if let Operand::Move(Place::Local(id)) = arg {
                        state.insert(*id);
                    }
                }
                state.remove(dest_id);
            }
            _ => {}
        }
    }
}


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
            Statement::Assign(Place::Local(id), rvalue, _) => {
                state.remove(id);
                match rvalue {
                    Rvalue::Use(op) | Rvalue::UnaryOp(_, op) => {
                        if let Operand::Copy(Place::Local(uid)) | Operand::Move(Place::Local(uid)) = op {
                            state.insert(*uid);
                        }
                    }
                    Rvalue::BinaryOp(_, l, r) => {
                        if let Operand::Copy(Place::Local(uid)) | Operand::Move(Place::Local(uid)) = l {
                            state.insert(*uid);
                        }
                        if let Operand::Copy(Place::Local(uid)) | Operand::Move(Place::Local(uid)) = r {
                            state.insert(*uid);
                        }
                    }
                    Rvalue::Ref(_, _, Place::Local(uid)) => {
                        state.insert(*uid);
                    }
                    Rvalue::Aggregate(ops) => {
                        for op in ops {
                            if let Operand::Copy(Place::Local(uid)) | Operand::Move(Place::Local(uid)) = op {
                                state.insert(*uid);
                            }
                        }
                    }
                    _ => {}
                }
            }
            Statement::StorageDead(id, _) | Statement::StorageLive(id, _) => {
                state.remove(id);
            }
            Statement::Drop(Place::Local(id), _) => {
                state.insert(*id);
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


#[derive(Clone, PartialEq)]
pub struct EverInitializedLocals {
    pub ever_init: HashSet<LocalId>,
}

impl DataflowAnalysis for EverInitializedLocals {
    type Domain = HashSet<LocalId>;

    fn is_forward(&self) -> bool { true }

    fn entry_state(&self, _body: &crate::MirBody) -> Self::Domain {
        self.ever_init.clone()
    }

    fn bottom_value(&self, _body: &crate::MirBody) -> Self::Domain {
        HashSet::new()
    }

    fn join(&self, state1: &mut Self::Domain, state2: &Self::Domain) -> bool {
        let mut changed = false;
        for &l in state2 {
            if state1.insert(l) {
                changed = true;
            }
        }
        changed
    }

    fn transfer_statement(&self, stmt: &crate::Statement, state: &mut Self::Domain) {
        if let crate::Statement::Assign(crate::Place::Local(id), _, _) = stmt {
            state.insert(*id);
        }
    }

    fn transfer_terminator(&self, _term: &crate::Terminator, _state: &mut Self::Domain) {}
}
