import os

with open('crates/apl_mir/src/lowering.rs', 'r', encoding='utf-8') as f:
    text = f.read()

new_cases = '''
            HirExprKind::Loop(body) => {
                let loop_block = self.new_block();
                let exit_block = self.new_block();
                
                self.blocks[self.current_block.0 as usize].terminator = Terminator::Goto(loop_block);
                self.current_block = loop_block;
                
                self.loop_scopes.push((loop_block, exit_block));
                self.lower_expr(body);
                self.loop_scopes.pop();
                
                self.blocks[self.current_block.0 as usize].terminator = Terminator::Goto(loop_block);
                self.current_block = exit_block;
                Rvalue::Use(Operand::Constant(Constant::Int(0, Type::Void)))
            }
            HirExprKind::Break => {
                if let Some(&(_, break_bb)) = self.loop_scopes.last() {
                    self.blocks[self.current_block.0 as usize].terminator = Terminator::Goto(break_bb);
                    self.current_block = self.new_block(); // dead code
                }
                Rvalue::Use(Operand::Constant(Constant::Int(0, Type::Void)))
            }
            HirExprKind::Continue => {
                if let Some(&(continue_bb, _)) = self.loop_scopes.last() {
                    self.blocks[self.current_block.0 as usize].terminator = Terminator::Goto(continue_bb);
                    self.current_block = self.new_block(); // dead code
                }
                Rvalue::Use(Operand::Constant(Constant::Int(0, Type::Void)))
            }
'''

# We also need to add loop_scopes to While loop!
while_start = text.find('HirExprKind::While { cond, body } => {')
if while_start != -1:
    body_eval = text.find('self.lower_expr(body);', while_start)
    if body_eval != -1:
        text = text[:body_eval] + 'self.loop_scopes.push((cond_block, exit_block));\n                ' + text[body_eval:]
        
    after_body = text.find('self.blocks[self.current_block.0 as usize].terminator = Terminator::Goto(cond_block);', body_eval)
    if after_body != -1:
        text = text[:after_body] + 'self.loop_scopes.pop();\n                ' + text[after_body:]

idx = text.find('            HirExprKind::While { cond, body } => {')
text = text[:idx] + new_cases + text[idx:]

with open('crates/apl_mir/src/lowering.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print("done")
