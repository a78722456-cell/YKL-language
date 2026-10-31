
import re

with open("crates/apl_sema/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

def replace_struct(match):
    return """            ExprKind::StructInit(name, fields) => {
                let mut resolved_name = name.clone();
                let mut decl_opt = self.structs.get(&resolved_name).cloned();
                if decl_opt.is_none() && !self.current_package.is_empty() {
                    resolved_name = format!("{}::{}", self.current_package, name);
                    decl_opt = self.structs.get(&resolved_name).cloned();
                }
                
                if let Some(decl) = decl_opt {
                    self.resolved_names.insert(expr.span, resolved_name.clone());
"""

content = re.sub(r"            ExprKind::StructInit\(name, fields\) => \{\n                if let Some\(decl\) = self\.structs\.get\(name\)\.cloned\(\) \{", replace_struct, content, flags=re.DOTALL)

with open("crates/apl_sema/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

