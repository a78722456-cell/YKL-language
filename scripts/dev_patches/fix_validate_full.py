import re

with open('crates/apl_mir/src/validate.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# find match stmt {
# up to the end of the fn validate_block
# I will just write the function explicitly

func = '''    fn validate_block(&self, block: &BasicBlock, local_ids: &HashSet<LocalId>, func_ids: &HashSet<FunctionId>) -> Result<(), String> {
        for stmt in &block.statements {
            match stmt {
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
            }
        }
        self.validate_terminator(&block.terminator, local_ids, func_ids)?;
        Ok(())
    }'''

text = re.sub(r'    fn validate_block.*?Ok\(\(\)\)\n    \}', func, text, flags=re.DOTALL)

with open('crates/apl_mir/src/validate.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print("done")
