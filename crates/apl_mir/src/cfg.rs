
use crate::*;
use std::collections::{HashMap, HashSet};

impl MirBody {
    pub fn successors(&self, block: BasicBlockId) -> Vec<BasicBlockId> {
        let b = &self.blocks[block.0 as usize];
        match &b.terminator {
            Terminator::Goto(target) => vec![*target],
            Terminator::SwitchInt { targets, otherwise, .. } => {
                let mut succ = Vec::new();
                for (_, t) in targets {
                    succ.push(*t);
                }
                succ.push(*otherwise);
                succ
            }
            Terminator::Call { target, .. } => vec![*target],
            Terminator::Drop { target, .. } => vec![*target],
            Terminator::Return(_) | Terminator::Unreachable | Terminator::Resume => vec![],
        }
    }

    pub fn predecessors(&self) -> HashMap<BasicBlockId, Vec<BasicBlockId>> {
        let mut preds = HashMap::new();
        for i in 0..self.blocks.len() {
            let id = BasicBlockId(i as u32);
            preds.insert(id, Vec::new());
        }

        for i in 0..self.blocks.len() {
            let id = BasicBlockId(i as u32);
            for succ in self.successors(id) {
                preds.get_mut(&succ).unwrap().push(id);
            }
        }
        preds
    }

    pub fn reachable_blocks(&self) -> HashSet<BasicBlockId> {
        let mut visited = HashSet::new();
        let mut stack = vec![self.start_block];
        
        while let Some(block) = stack.pop() {
            if visited.insert(block) {
                for succ in self.successors(block) {
                    stack.push(succ);
                }
            }
        }
        visited
    }
}

