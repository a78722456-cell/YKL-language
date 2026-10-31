with open('crates/apl_hir/src/lowering.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# We need to replace the dummy lower_expr with a real one.
# It's better to just output a python script that completely rewrites lower_expr.
