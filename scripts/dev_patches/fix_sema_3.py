
with open("crates/apl_sema/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

missing = """
    pub fn analyze_program(&mut self, prog: &Program) -> Result<(), Vec<SemaError>> {
        for item in &prog.items {
            match item {
                Item::Function(func) => {
                    self.functions.insert(func.name.clone(), func.clone());
                }
                Item::Struct(st) => {
                    self.structs.insert(st.name.clone(), st.clone());
                }
                Item::Enum(en) => {
                    self.enums.insert(en.name.clone(), en.clone());
                }
                Item::ExternBlock(decls, _) => {
                    for decl in decls {
                        self.extern_fns.insert(decl.name.clone(), decl.clone());
                    }
                }
                Item::Impl(imp) => {
                    for mut method in imp.methods.clone() {
                        method.name = format!("{}_{}", imp.target_type, method.name);
                        self.functions.insert(method.name.clone(), method);
                    }
                }
                _ => {}
            }
        }
        for item in &prog.items {
            if let Item::Function(func) = item {
                self.analyze_function(func);
            }
            if let Item::Impl(imp) = item {
                for method in &imp.methods {
                    self.analyze_function(method);
                }
            }
        }
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }
"""

start_idx = content.find("    pub fn analyze_expr(&mut self, expr: &Expr) -> Type {")
if start_idx != -1:
    content = content[:start_idx] + missing + content[start_idx:]

with open("crates/apl_sema/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

