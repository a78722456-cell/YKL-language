import os

with open('crates/apl_mir/src/dataflow.rs', 'r', encoding='utf-8') as f:
    text = f.read()

backward_code = '''
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
'''

text = text.replace(
'''            } else {
                // backward analysis can be added later
                unimplemented!("Backward dataflow not yet implemented");
            }''', backward_code)

with open('crates/apl_mir/src/dataflow.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print("done")
