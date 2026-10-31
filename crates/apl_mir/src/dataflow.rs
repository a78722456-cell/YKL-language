
use crate::*;
use std::collections::HashMap;

pub trait DataflowAnalysis {
    type Domain: Clone + PartialEq;
    
    // Direction of the analysis
    fn is_forward(&self) -> bool { true }

    // Initial state for the entry block
    fn entry_state(&self, body: &MirBody) -> Self::Domain;
    
    // Initial state for all other blocks
    fn bottom_value(&self, body: &MirBody) -> Self::Domain;

    // Join operator (e.g. union for may-analysis, intersection for must-analysis)
    fn join(&self, state1: &mut Self::Domain, state2: &Self::Domain) -> bool; // returns true if changed

    // Transfer function for a single statement
    fn transfer_statement(&self, stmt: &Statement, state: &mut Self::Domain);

    // Transfer function for a terminator
    fn transfer_terminator(&self, term: &Terminator, state: &mut Self::Domain);
}

pub struct DataflowResults<T> {
    pub entry_sets: HashMap<BasicBlockId, T>,
    pub exit_sets: HashMap<BasicBlockId, T>,
}

pub fn run_analysis<A: DataflowAnalysis>(body: &MirBody, analysis: &A) -> DataflowResults<A::Domain> {
    let mut entry_sets = HashMap::new();
    let mut exit_sets = HashMap::new();

    for i in 0..body.blocks.len() {
        let id = BasicBlockId(i as u32);
        entry_sets.insert(id, analysis.bottom_value(body));
        exit_sets.insert(id, analysis.bottom_value(body));
    }

    if analysis.is_forward() {
        entry_sets.insert(body.start_block, analysis.entry_state(body));
    }

    let predecessors = body.predecessors();
    let mut changed = true;

    while changed {
        changed = false;

        for i in 0..body.blocks.len() {
            let block_id = BasicBlockId(i as u32);
            
            // Join predecessors
            if analysis.is_forward() {
                let mut current_entry = entry_sets[&block_id].clone();
                if block_id != body.start_block {
                    if let Some(preds) = predecessors.get(&block_id) {
                        for p in preds {
                            analysis.join(&mut current_entry, &exit_sets[p]);
                        }
                    }
                }
                entry_sets.insert(block_id, current_entry.clone());

                let mut current_exit = current_entry;
                let block = &body.blocks[i];
                for stmt in &block.statements {
                    analysis.transfer_statement(stmt, &mut current_exit);
                }
                analysis.transfer_terminator(&block.terminator, &mut current_exit);

                if current_exit != exit_sets[&block_id] {
                    exit_sets.insert(block_id, current_exit);
                    changed = true;
                }

            } else {
                let successors = body.successors(block_id);
                let mut current_exit = exit_sets[&block_id].clone();
                for s in successors {
                    analysis.join(&mut current_exit, &entry_sets[&s]);
                }
                exit_sets.insert(block_id, current_exit.clone());

                let mut current_entry = current_exit;
                let block = &body.blocks[i];
                
                analysis.transfer_terminator(&block.terminator, &mut current_entry);
                for stmt in block.statements.iter().rev() {
                    analysis.transfer_statement(stmt, &mut current_entry);
                }

                if current_entry != entry_sets[&block_id] {
                    entry_sets.insert(block_id, current_entry);
                    changed = true;
                }
            }

        }
    }

    DataflowResults {
        entry_sets,
        exit_sets,
    }
}

