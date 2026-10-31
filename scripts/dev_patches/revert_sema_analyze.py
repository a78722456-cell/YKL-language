
import re

with open("crates/apl_sema/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

analyze_old = """    pub fn analyze_program(&mut self, prog: &Program) -> Result<(), Vec<SemaError>> {
        for item in &prog.items {
            match item {
                Item::Package(name, _) => {
                    self.current_package = name.clone();
                }
                Item::Function(func) => {
                    let mut cloned = func.clone();
                    let fqdn = if self.current_package.is_empty() || func.name == "main" {
                        func.name.clone()
                    } else {
                        format!("{}::{}", self.current_package, func.name)
                    };
                    cloned.name = fqdn.clone();
                    self.functions.insert(fqdn, cloned);
                }
                Item::Struct(st) => {
                    let mut cloned = st.clone();
                    let fqdn = if self.current_package.is_empty() {
                        st.name.clone()
                    } else {
                        format!("{}::{}", self.current_package, st.name)
                    };
                    cloned.name = fqdn.clone();
                    self.structs.insert(fqdn, cloned);
                }
                Item::Enum(en) => {
                    let mut cloned = en.clone();
                    let fqdn = if self.current_package.is_empty() {
                        en.name.clone()
                    } else {
                        format!("{}::{}", self.current_package, en.name)
                    };
                    cloned.name = fqdn.clone();
                    self.enums.insert(fqdn, cloned);
                }
                Item::ExternBlock(decls, _) => {
                    for decl in decls {
                        let mut cloned = decl.clone();
                        let fqdn = if self.current_package.is_empty() {
                            decl.name.clone()
                        } else {
                            format!("{}::{}", self.current_package, decl.name)
                        };
                        cloned.name = fqdn.clone();
                        self.extern_fns.insert(fqdn, cloned);
                    }
                }
                Item::Impl(imp) => {
                    for mut method in imp.methods.clone() {
                        method.name = format!("{}_{}", imp.target_type, method.name);
                        let fqdn = if self.current_package.is_empty() {
                            method.name.clone()
                        } else {
                            format!("{}::{}", self.current_package, method.name)
                        };
                        method.name = fqdn.clone();
                        self.functions.insert(fqdn, method);
                    }
                }
                _ => {}
            }
        }
        self.current_package.clear();
        for item in &prog.items {
            if let Item::Package(name, _) = item {
                self.current_package = name.clone();
            }
            if let Item::Function(func) = item {
                let mut cloned = func.clone();
                if !self.current_package.is_empty() && func.name != "main" {
                    cloned.name = format!("{}::{}", self.current_package, func.name);
                }
                self.analyze_function(&cloned);
            }
            if let Item::Impl(imp) = item {
                for method in &imp.methods {
                    let mut cloned = method.clone();
                    cloned.name = format!("{}_{}", imp.target_type, method.name);
                    if !self.current_package.is_empty() {
                        cloned.name = format!("{}::{}", self.current_package, cloned.name);
                    }
                    self.analyze_function(&cloned);
                }
            }
        }"""

analyze_new = """    pub fn analyze_program(&mut self, prog: &Program) -> Result<(), Vec<SemaError>> {
        for item in &prog.items {
            match item {
                Item::Package(name, _) => {
                    self.current_package = name.clone();
                }
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
        self.current_package.clear();
        for item in &prog.items {
            if let Item::Package(name, _) = item {
                self.current_package = name.clone();
            }
            if let Item::Function(func) = item {
                self.analyze_function(func);
            }
            if let Item::Impl(imp) = item {
                for method in &imp.methods {
                    self.analyze_function(method);
                }
            }
        }"""

content = content.replace(analyze_old, analyze_new)

with open("crates/apl_sema/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

