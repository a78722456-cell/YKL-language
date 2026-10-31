import os
import re

with open('crates/apl_mir/src/analysis.rs', 'r', encoding='utf-8') as f:
    text = f.read()

ever_init = '''
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
'''

text += "\n" + ever_init

with open('crates/apl_mir/src/analysis.rs', 'w', encoding='utf-8') as f:
    f.write(text)

with open('crates/apl_borrowck/src/lib.rs', 'r', encoding='utf-8') as f:
    text2 = f.read()

# I will update check_body to run EverInitializedLocals
# and check_moves to use it.
text2 = text2.replace('use apl_mir::analysis::{LivenessAnalysis, MaybeUninitializedLocals};', 'use apl_mir::analysis::{LivenessAnalysis, MaybeUninitializedLocals, EverInitializedLocals};')

text2 = text2.replace('let uninit_res = run_analysis(body, &uninit);', 'let uninit_res = run_analysis(body, &uninit);\n    let ever_init = EverInitializedLocals { ever_init: HashSet::new() };\n    let ever_init_res = run_analysis(body, &ever_init);')

text2 = text2.replace('check_moves(body, &uninit_res, &mut errors);', 'check_moves(body, &uninit_res, &ever_init_res, &mut errors);')

text2 = text2.replace('fn check_moves(body: &MirBody, uninit_res: &DataflowResults<HashSet<apl_mir::LocalId>>, errors: &mut Vec<BorrowckError>) {', 'fn check_moves(body: &MirBody, uninit_res: &DataflowResults<HashSet<apl_hir::LocalId>>, ever_init_res: &DataflowResults<HashSet<apl_hir::LocalId>>, errors: &mut Vec<BorrowckError>) {')

text2 = text2.replace('fn check_rvalue_uses(rvalue: &Rvalue, uninit: &HashSet<apl_mir::LocalId>, span: Span, errors: &mut Vec<BorrowckError>) {', 'fn check_rvalue_uses(rvalue: &Rvalue, uninit: &HashSet<apl_hir::LocalId>, ever_init: &HashSet<apl_hir::LocalId>, span: Span, errors: &mut Vec<BorrowckError>) {')

text2 = text2.replace('check_rvalue_uses(rvalue, &current_uninit, *span, errors);', 'check_rvalue_uses(rvalue, &current_uninit, &current_ever, *span, errors);')

# In check_moves:
replace_check_moves = '''fn check_moves(body: &MirBody, uninit_res: &DataflowResults<HashSet<apl_hir::LocalId>>, ever_init_res: &DataflowResults<HashSet<apl_hir::LocalId>>, errors: &mut Vec<BorrowckError>) {
    for (bb_idx, block) in body.blocks.iter().enumerate() {
        let bb = BasicBlockId(bb_idx as u32);
        let mut current_uninit = uninit_res.entry_sets[&bb].clone();
        let mut current_ever = ever_init_res.entry_sets[&bb].clone();
        
        for stmt in &block.statements {
            match stmt {
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
                _ => {}
            }
        }
    }
}'''

text2 = re.sub(r'fn check_moves\(body: &MirBody.*?\n\}\n\}', replace_check_moves, text2, flags=re.DOTALL)

# In check_rvalue_uses:
replace_check_rvalue = '''fn check_rvalue_uses(rvalue: &Rvalue, uninit: &HashSet<apl_hir::LocalId>, ever_init: &HashSet<apl_hir::LocalId>, span: Span, errors: &mut Vec<BorrowckError>) {
    let mut check_op = |op: &Operand| {
        if let Operand::Copy(Place::Local(id)) | Operand::Move(Place::Local(id)) = op {
            if uninit.contains(id) {
                if ever_init.contains(id) {
                    errors.push(BorrowckError::UseAfterMove { span });
                } else {
                    errors.push(BorrowckError::UseOfUninitializedVariable { span });
                }
            }
        }
    };
    match rvalue {
        Rvalue::Use(op) | Rvalue::UnaryOp(_, op) => check_op(op),
        Rvalue::BinaryOp(_, l, r) => { check_op(l); check_op(r); }
        Rvalue::Ref(_, _, Place::Local(id)) => {
            if uninit.contains(id) {
                if ever_init.contains(id) {
                    errors.push(BorrowckError::UseAfterMove { span });
                } else {
                    errors.push(BorrowckError::UseOfUninitializedVariable { span });
                }
            }
        }
        _ => {}
    }
}'''

text2 = re.sub(r'fn check_rvalue_uses\(rvalue: &Rvalue.*?\n\}\n\}', replace_check_rvalue, text2, flags=re.DOTALL)

with open('crates/apl_borrowck/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(text2)

print("done")
