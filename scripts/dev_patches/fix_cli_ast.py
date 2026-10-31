
import re

with open("crates/apl_cli/src/main.rs", "r", encoding="utf-8") as f:
    content = f.read()

target = """    let mut sema = SemanticAnalyzer::new();"""
new_target = """    let mut current_package = String::new();
    for item in &mut prog.items {
        if let apl_ast::Item::Package(name, _) = item {
            current_package = name.clone();
        } else if !current_package.is_empty() {
            match item {
                apl_ast::Item::Function(func) => {
                    if func.name != "main" {
                        func.name = format!("{}::{}", current_package, func.name);
                    }
                }
                apl_ast::Item::Struct(st) => {
                    st.name = format!("{}::{}", current_package, st.name);
                }
                apl_ast::Item::Enum(en) => {
                    en.name = format!("{}::{}", current_package, en.name);
                }
                apl_ast::Item::ExternBlock(decls, _) => {
                    for decl in decls {
                        decl.name = format!("{}::{}", current_package, decl.name);
                    }
                }
                apl_ast::Item::Impl(imp) => {
                    for method in &mut imp.methods {
                        method.name = format!("{}::{}", current_package, method.name);
                    }
                }
                _ => {}
            }
        }
    }

    let mut sema = SemanticAnalyzer::new();"""

content = content.replace(target, new_target)

with open("crates/apl_cli/src/main.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

