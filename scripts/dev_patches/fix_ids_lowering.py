with open('crates/apl_hir/src/lowering.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace('pub name: String,', 'pub id: LocalId,\n    pub name: String,')
text = text.replace('pub fn define_var(&mut self, name: String, ty: Type, mutability: bool, span: Span) -> Result<(), ()> {', 'pub fn define_var(&mut self, name: String, ty: Type, mutability: bool, span: Span) -> Result<LocalId, ()> {')

# We need a proper LocalId allocator.
# This requires a bit of manual editing.
