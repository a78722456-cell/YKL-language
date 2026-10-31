import os

# Add to HirExprKind
with open('crates/apl_hir/src/lib.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace(
'''    Block(HirBlock, Option<Box<HirExpr>>),
}''',
'''    Block(HirBlock, Option<Box<HirExpr>>),
    Loop(Box<HirExpr>),
    Break,
    Continue,
}'''
)
with open('crates/apl_hir/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(text)

# Add to lower_expr
with open('crates/apl_hir/src/lowering.rs', 'r', encoding='utf-8') as f:
    text = f.read()

new_cases = '''
            ExprKind::Loop(body) => {
                let h_body = self.lower_expr(body);
                HirExpr {
                    kind: HirExprKind::Loop(Box::new(h_body)),
                    ty: Type::Void,
                    span: expr.span,
                }
            }
            ExprKind::Break => {
                HirExpr {
                    kind: HirExprKind::Break,
                    ty: Type::Void,
                    span: expr.span,
                }
            }
            ExprKind::Continue => {
                HirExpr {
                    kind: HirExprKind::Continue,
                    ty: Type::Void,
                    span: expr.span,
                }
            }
'''

idx = text.find('            ExprKind::While(cond, body) => {')
text = text[:idx] + new_cases + text[idx:]

with open('crates/apl_hir/src/lowering.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print("done")
