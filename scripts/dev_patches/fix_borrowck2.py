import re

with open('crates/apl_borrowck/src/lib.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# I will find the exact bounds of the functions
start1 = text.find('fn check_moves(')
end1 = text.find('fn check_borrows(')

replacement = '''fn check_moves(body: &MirBody, uninit_res: &DataflowResults<HashSet<apl_hir::LocalId>>, ever_init_res: &DataflowResults<HashSet<apl_hir::LocalId>>, errors: &mut Vec<BorrowckError>) {
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
}

fn check_rvalue_uses(rvalue: &Rvalue, uninit: &HashSet<apl_hir::LocalId>, ever_init: &HashSet<apl_hir::LocalId>, span: Span, errors: &mut Vec<BorrowckError>) {
    let mut check_op = |op: &Operand| {
        if let Operand::Copy(Place::Local(id)) | Operand::Move(Place::Local(id)) = op {
            if uninit.contains(id) {
                if ever_init.contains(id) {
                    errors.push(BorrowckError::UseAfterMove { span: *span });
                } else {
                    errors.push(BorrowckError::UseOfUninitializedVariable { span: *span });
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
}

'''

text = text[:start1] + replacement + text[end1:]

with open('crates/apl_borrowck/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("done")
