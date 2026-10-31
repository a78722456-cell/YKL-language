
with open("crates/apl_mir/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

call_old = """                if let ExprKind::Ident(name) = &callee.kind {
                    self.push_inst(MirInstruction::Call(dest, name.clone(), args));
                }"""

call_new = """                if let ExprKind::Ident(name) = &callee.kind {
                    let resolved = self.resolved_names.get(&callee.span).unwrap_or(name);
                    self.push_inst(MirInstruction::Call(dest, resolved.clone(), args));
                }"""

content = content.replace(call_old, call_new)

with open("crates/apl_mir/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

