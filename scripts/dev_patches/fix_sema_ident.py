
with open("crates/apl_sema/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

ident_old = """                if let ExprKind::Ident(name) = &callee.kind {
                    if let Some(func) = self.functions.get(name).cloned() {"""

ident_new = """                if let ExprKind::Ident(name) = &callee.kind {
                    let mut resolved_name = name.clone();
                    let mut func_opt = self.functions.get(&resolved_name).cloned();
                    if func_opt.is_none() && !self.current_package.is_empty() {
                        resolved_name = format!("{}::{}", self.current_package, name);
                        func_opt = self.functions.get(&resolved_name).cloned();
                    }
                    if func_opt.is_none() {
                        // Check if it is a method without full namespace (e.g., Vec_new in std::vec)
                        // This might be overly simple, but covers the basic namespace resolution
                    }

                    if let Some(func) = func_opt {
                        self.resolved_names.insert(callee.span, resolved_name.clone());"""

content = content.replace(ident_old, ident_new)

# Also need to resolve `ExternBlock` calls!
extern_old = """                    } else if let Some(ext) = self.extern_fns.get(name).cloned() {"""
extern_new = """                    } else {
                        let mut resolved_name = name.clone();
                        let mut ext_opt = self.extern_fns.get(&resolved_name).cloned();
                        if ext_opt.is_none() && !self.current_package.is_empty() {
                            resolved_name = format!("{}::{}", self.current_package, name);
                            ext_opt = self.extern_fns.get(&resolved_name).cloned();
                        }
                        if let Some(ext) = ext_opt {
                            self.resolved_names.insert(callee.span, resolved_name.clone());"""

content = content.replace(extern_old, extern_new)

# Since I modified the `if else` structure, let's fix it with regex or targeted replace:
# Let's just rewrite `ExprKind::Call` handling for `Ident(name)`.

