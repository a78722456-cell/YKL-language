pub mod points_to;

use apl_mir::{MirBody, BasicBlockId, Statement, Place, Operand, Rvalue, BorrowKind};
use apl_mir::analysis::{LivenessAnalysis, MaybeUninitializedLocals, EverInitializedLocals};
use apl_mir::dataflow::{run_analysis, DataflowResults, DataflowAnalysis};
use apl_ast::Span;
use std::collections::HashSet;
use points_to::PointsToAnalysis;

#[derive(Debug)]
pub enum BorrowckError {
    UseOfUninitializedVariable { span: Span },
    UseAfterMove { span: Span },
    MutableBorrowConflict { span: Span },
    ImmutableBorrowConflict { span: Span },
    MutationWhileBorrowed { span: Span },
}

pub fn check_body(body: &MirBody) -> Vec<BorrowckError> {
    let mut errors = Vec::new();
    
    // 1. Run Analyses
    let liveness = LivenessAnalysis;
    let liveness_res = run_analysis(body, &liveness);
    
    let uninit = MaybeUninitializedLocals { uninit: HashSet::new() };
    let uninit_res = run_analysis(body, &uninit);
    let ever_init = EverInitializedLocals { ever_init: HashSet::new() };
    let ever_init_res = run_analysis(body, &ever_init);
    
    let points_to = PointsToAnalysis::new(body);
    let points_to_res = run_analysis(body, &points_to);

    // 2. Check for Move / Initialization errors
    check_moves(body, &uninit_res, &ever_init_res, &mut errors);

    // 3. Check for Borrow errors
    check_borrows(body, &points_to, &points_to_res, &liveness, &liveness_res, &mut errors);

    errors
}

fn check_moves(body: &MirBody, uninit_res: &DataflowResults<HashSet<apl_hir::LocalId>>, ever_init_res: &DataflowResults<HashSet<apl_hir::LocalId>>, errors: &mut Vec<BorrowckError>) {
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
                Statement::Drop(Place::Local(id), _) => {
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
        Rvalue::Aggregate(ops) => { for op in ops { check_op(op); } },
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

fn check_borrows(
    body: &MirBody,
    points_to: &PointsToAnalysis,
    points_to_res: &DataflowResults<points_to::PointsToDomain>,
    liveness: &LivenessAnalysis,
    liveness_res: &DataflowResults<HashSet<apl_hir::LocalId>>,
    errors: &mut Vec<BorrowckError>
) {
    for (bb_idx, block) in body.blocks.iter().enumerate() {
        let bb = BasicBlockId(bb_idx as u32);
        
        // Compute Liveness at each statement
        let mut live_at_stmt = Vec::new();
        let mut current_live = liveness_res.exit_sets[&bb].clone();
        
        liveness.transfer_terminator(&block.terminator, &mut current_live);
        for stmt in block.statements.iter().rev() {
            live_at_stmt.push(current_live.clone());
            liveness.transfer_statement(stmt, &mut current_live);
        }
        live_at_stmt.reverse();
        
        // Walk forward
        let mut current_points = points_to_res.entry_sets[&bb].clone();
        for (i, stmt) in block.statements.iter().enumerate() {
            let live_here = &live_at_stmt[i];
            
            // Active borrows = union of points-to for all LIVE variables
            let mut active_borrows = HashSet::new();
            for live_local in live_here {
                if let Some(borrows) = current_points.map.get(live_local) {
                    for b in borrows {
                        active_borrows.insert(*b);
                    }
                }
            }
            
            // Check conflicts
            if let Statement::Assign(dest_place, rvalue, span) = stmt {
                // 1. Mutation of borrowed place
                for b_id in &active_borrows {
                    let b = &points_to.borrows[b_id.0 as usize];
                    if b.borrowed_place == *dest_place {
                        errors.push(BorrowckError::MutationWhileBorrowed { span: *span });
                    }
                }
                
                // 2. Conflicting borrows
                if let Rvalue::Ref(_, new_kind, ref new_place) = rvalue {
                    for b_id in &active_borrows {
                        let b = &points_to.borrows[b_id.0 as usize];
                        if b.borrowed_place == *new_place {
                            if *new_kind == BorrowKind::Mut || b.kind == BorrowKind::Mut {
                                if *new_kind == BorrowKind::Mut {
                                    errors.push(BorrowckError::MutableBorrowConflict { span: *span });
                                } else {
                                    errors.push(BorrowckError::ImmutableBorrowConflict { span: *span });
                                }
                            }
                        }
                    }
                }
            }
            
            points_to.transfer_statement(stmt, &mut current_points);
        }
    }
}
