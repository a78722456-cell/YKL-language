import os

with open('crates/apl_mir/src/lowering.rs', 'r', encoding='utf-8') as f:
    text = f.read()

new_funcs = '''
    fn make_temp_uninit(&mut self, ty: Type, span: Span) -> Place {
        let id = LocalId(1000 + self.locals.len() as u32);
        self.add_local(id, ty, span, true);
        self.blocks[self.current_block.0 as usize].statements.push(Statement::StorageLive(id, span));
        Place::Local(id)
    }

    fn make_temp(&mut self, rvalue: Rvalue, span: Span, ty: Type) -> Place {
'''
text = text.replace('    fn make_temp(&mut self, rvalue: Rvalue, span: Span, ty: Type) -> Place {', new_funcs)

with open('crates/apl_mir/src/lowering.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print("done")
