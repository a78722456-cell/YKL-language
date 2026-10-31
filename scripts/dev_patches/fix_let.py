import os

with open('crates/apl_hir/src/lowering.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace(
'''                let hir_init = if let Some(e) = init {
                    Some(self.lower_expr(e.as_ref()))
                } else {
                    None
                };''',
'''                let hir_init = Some(self.lower_expr(init));'''
)
with open('crates/apl_hir/src/lowering.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print('done')
