use std::collections::{HashMap, HashSet};
use apl_mir::{MirBody, Statement, Terminator, Rvalue, Operand, Place, BorrowKind};
use apl_mir::dataflow::DataflowAnalysis;
use apl_hir::LocalId;
use apl_ast::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BorrowId(pub u32);

#[derive(Debug, Clone)]
pub struct BorrowInfo {
    pub id: BorrowId,
    pub kind: BorrowKind,
    pub borrowed_place: Place,
    pub span: Span,
}

#[derive(Clone, PartialEq)]
pub struct PointsToDomain {
    pub map: HashMap<LocalId, HashSet<BorrowId>>,
}

pub struct PointsToAnalysis {
    pub borrows: Vec<BorrowInfo>,
}

impl PointsToAnalysis {
    pub fn new(body: &MirBody) -> Self {
        let mut borrows = Vec::new();
        let mut next_id = 0;
        
        // Pre-pass to gather all borrow creations
        for block in &body.blocks {
            for stmt in &block.statements {
                if let Statement::Assign(_, Rvalue::Ref(_, kind, place), span) = stmt {
                    borrows.push(BorrowInfo {
                        id: BorrowId(next_id),
                        kind: kind.clone(),
                        borrowed_place: place.clone(),
                        span: *span,
                    });
                    next_id += 1;
                }
            }
        }
        
        Self { borrows }
    }
}

impl DataflowAnalysis for PointsToAnalysis {
    type Domain = PointsToDomain;

    fn is_forward(&self) -> bool { true }

    fn entry_state(&self, _body: &MirBody) -> Self::Domain {
        PointsToDomain { map: HashMap::new() }
    }

    fn bottom_value(&self, _body: &MirBody) -> Self::Domain {
        PointsToDomain { map: HashMap::new() }
    }

    fn join(&self, state1: &mut Self::Domain, state2: &Self::Domain) -> bool {
        let mut changed = false;
        for (local, set2) in &state2.map {
            let set1 = state1.map.entry(*local).or_insert_with(HashSet::new);
            let prev_len = set1.len();
            for b in set2 {
                set1.insert(*b);
            }
            if set1.len() > prev_len {
                changed = true;
            }
        }
        changed
    }

    fn transfer_statement(&self, stmt: &Statement, state: &mut Self::Domain) {
        if let Statement::Assign(Place::Local(dest), rvalue, span) = stmt {
            match rvalue {
                Rvalue::Ref(_, _, _) => {
                    // Find the borrow id
                    let mut b_id = None;
                    for b in &self.borrows {
                        if b.span == *span { // A bit hacky, but works for this simplified version
                            b_id = Some(b.id);
                            break;
                        }
                    }
                    if let Some(id) = b_id {
                        let mut set = HashSet::new();
                        set.insert(id);
                        state.map.insert(*dest, set);
                    }
                }
                Rvalue::Use(Operand::Copy(Place::Local(src))) | Rvalue::Use(Operand::Move(Place::Local(src))) => {
                    if let Some(src_set) = state.map.get(src).cloned() {
                        state.map.insert(*dest, src_set);
                    } else {
                        state.map.remove(dest);
                    }
                }
                _ => {
                    state.map.remove(dest); // Assigning something else clears the borrows
                }
            }
        }
    }

    fn transfer_terminator(&self, term: &Terminator, state: &mut Self::Domain) {
        if let Terminator::Call { args, .. } = term {
            for arg in args {
                if let Operand::Copy(Place::Local(id)) | Operand::Move(Place::Local(id)) = arg {
                    state.map.remove(id);
                }
            }
        }
    }
}
