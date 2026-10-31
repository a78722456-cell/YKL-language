import re

with open('crates/apl_hir/src/lowering.rs', 'r', encoding='utf-8') as f:
    text = f.read()

replacement = '''            Stmt::Let { name, ty, mutability, init, span } => {
                let hir_init = self.lower_expr(init);
                let defined_ty = ty.clone().unwrap_or(hir_init.ty.clone());
                let id = self.define_var(name.clone(), defined_ty.clone(), *mutability, *span).unwrap_or(LocalId(0));
                Some(HirStmt::Let {
                    id,
                    name: name.clone(),
                    mutability: *mutability,
                    ty: defined_ty,
                    init: Some(hir_init),
                    span: *span,
                })
            }'''

text = re.sub(
    r'Stmt::Let \{ name, ty, mutability, init, span \} => \{\s*let defined_ty = ty\.clone\(\)\.unwrap_or\(Type::Void\);\s*let hir_init = Some\(self\.lower_expr\(init\)\);\s*let id = self\.define_var\(name\.clone\(\), defined_ty\.clone\(\), \*mutability, \*span\)\.unwrap_or\(LocalId\(0\)\);\s*Some\(HirStmt::Let \{\s*id,\s*name: name\.clone\(\),\s*mutability: \*mutability,\s*ty: defined_ty,\s*init: hir_init,\s*span: \*span,\s*\}\)\s*\}',
    replacement,
    text
)

with open('crates/apl_hir/src/lowering.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("done")
