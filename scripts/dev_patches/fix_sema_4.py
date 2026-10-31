
with open("crates/apl_sema/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

missing = """
    pub fn analyze_function(&mut self, func: &Function) {
        self.scopes.push(Scope { vars: HashMap::new() });
        for param in &func.params {
            self.define_var(param.name.clone(), param.ty.clone(), false, param.span).unwrap();
        }
        self.current_function_return_type = Some(func.return_type.clone());
        let ty = self.analyze_expr(&func.body);
        
        if func.return_type != Type::Void && !self.is_compatible(&func.return_type, &ty) {
            self.errors.push(SemaError::TypeMismatch(func.return_type.clone(), ty, func.body.span));
        }

        let scope = self.scopes.pop().unwrap();
        for (_, var_info) in scope.vars {
            if !var_info.is_used && var_info.name != "_" {
                self.warnings.push(SemaWarning::UnusedVariable(var_info.name, var_info.span));
            }
        }
        self.current_function_return_type = None;
    }
"""

start_idx = content.find("    pub fn analyze_program")
if start_idx != -1:
    content = content[:start_idx] + missing + content[start_idx:]

with open("crates/apl_sema/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

