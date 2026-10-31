import os

with open('crates/apl_mir/src/lib.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace('pub mod validate;', 'pub mod validate;\npub mod analysis;')

text = text.replace(
'''pub struct LocalDecl {
    pub id: LocalId,
    pub ty: Type,
    pub span: Span,
    pub mutability: bool,
}''',
'''pub struct LocalDecl {
    pub id: LocalId,
    pub ty: Type,
    pub span: Span,
    pub mutability: bool,
    pub is_param: bool,
}'''
)

with open('crates/apl_mir/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(text)

with open('crates/apl_mir/src/lowering.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace(
'''    pub fn add_local(&mut self, id: LocalId, ty: Type, span: Span, mutability: bool) {
        // We might have gaps if LocalId is global, but usually we just push and use the ID as an index or map it.
        // For simplicity, let's assume locals map linearly or we just store them.
        self.locals.push(LocalDecl { id, ty, span, mutability });
    }''',
'''    pub fn add_local(&mut self, id: LocalId, ty: Type, span: Span, mutability: bool, is_param: bool) {
        self.locals.push(LocalDecl { id, ty, span, mutability, is_param });
    }'''
)

# Replace all calls to add_local
text = text.replace('self.add_local(param.id, param.ty.clone(), param.span, false);', 'self.add_local(param.id, param.ty.clone(), param.span, false, true);')
text = text.replace('self.add_local(*id, ty.clone(), *span, *mutability);', 'self.add_local(*id, ty.clone(), *span, *mutability, false);')
text = text.replace('self.add_local(id, ty, span, true);', 'self.add_local(id, ty, span, true, false);')

with open('crates/apl_mir/src/lowering.rs', 'w', encoding='utf-8') as f:
    f.write(text)

# Fix analysis entry_state
with open('crates/apl_mir/src/analysis.rs', 'r', encoding='utf-8') as f:
    text = f.read()
    
text = text.replace(
'''        for local in &body.locals {
            // Assume params are initialized, others are uninitialized
            // Wait, we don't have a direct flag for param in LocalDecl yet,
            // but in MirBuilder we add params first. Let's assume locals without init are uninit.
            // Actually, all non-parameter locals are uninitialized at entry.
            // Let's assume LocalId < body.params_len are params? We don't have params_len.
            // For now, let's just mark everything uninitialized except the first N params.
            // We can just add a is_param: bool to LocalDecl!
        }''',
'''        for local in &body.locals {
            if !local.is_param {
                state.insert(local.id);
            }
        }'''
)
with open('crates/apl_mir/src/analysis.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("done")
