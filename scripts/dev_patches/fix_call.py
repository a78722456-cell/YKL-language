
import re

with open("crates/apl_sema/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

def replace_call(match):
    return """            ExprKind::Call(callee, args) => {
                let mut arg_types = Vec::new();
                for arg in args {
                    arg_types.push(self.analyze_expr(arg));
                }
                if let ExprKind::Ident(name) = &callee.kind {
                    let mut resolved_name = name.clone();
                    let mut func_opt = self.functions.get(&resolved_name).cloned();
                    if func_opt.is_none() && !self.current_package.is_empty() {
                        resolved_name = format!("{}::{}", self.current_package, name);
                        func_opt = self.functions.get(&resolved_name).cloned();
                    }

                    if let Some(func) = func_opt {
                        self.resolved_names.insert(callee.span, resolved_name.clone());
                        let mut type_mapping = HashMap::new();

                        // Infer generics from arguments
                        for (i, param) in func.params.iter().enumerate() {
                            if let Some(actual_ty) = arg_types.get(i) {
                                Self::infer_generics(
                                    &param.ty,
                                    actual_ty,
                                    &mut type_mapping,
                                    &func.generics,
                                );
                            }
                        }

                        // Check arguments
                        for (i, param) in func.params.iter().enumerate() {
                            if let Some(actual_ty) = arg_types.get(i) {
                                let subst_expected =
                                    self.monomorphize_type(&param.ty, &type_mapping);
                                if !self.is_compatible(&subst_expected, actual_ty) {
                                    self.errors.push(SemaError::TypeMismatch(
                                        subst_expected,
                                        actual_ty.clone(),
                                        expr.span,
                                    ));
                                }
                            }
                        }

                        if args.len() != func.params.len() {
                            self.errors.push(SemaError::TypeMismatch(
                                Type::Void,
                                Type::Void,
                                expr.span,
                            )); // Using TypeMismatch as placeholder for WrongArgCount for now
                        }

                        return self.monomorphize_type(&func.return_type, &type_mapping);
                    } else {
                        let mut ext_opt = self.extern_fns.get(&resolved_name).cloned();
                        if ext_opt.is_none() && !self.current_package.is_empty() {
                            let temp_res = format!("{}::{}", self.current_package, name);
                            if self.extern_fns.contains_key(&temp_res) {
                                resolved_name = temp_res;
                                ext_opt = self.extern_fns.get(&resolved_name).cloned();
                            }
                        }
                        
                        if let Some(ext) = ext_opt {
                            self.resolved_names.insert(callee.span, resolved_name.clone());
                            for (i, param) in ext.params.iter().enumerate() {
                                if let Some(actual_ty) = arg_types.get(i) {
                                    if !self.is_compatible(&param.1, actual_ty) {
                                        self.errors.push(SemaError::TypeMismatch(
                                            param.1.clone(),
                                            actual_ty.clone(),
                                            expr.span,
                                        ));
                                    }
                                }
                            }
                            if args.len() != ext.params.len() {
                                self.errors.push(SemaError::TypeMismatch(Type::Void, Type::Void, expr.span));
                            }
                            return ext.return_type;
                        } else {
                            if let Some((en_name, var_name)) = name.split_once("::") {
"""

content = re.sub(r"            ExprKind::Call\(callee, args\) => \{.*?if let Some\(\(en_name, var_name\)\) = name\.split_once\(\"::\"\) \{", replace_call, content, flags=re.DOTALL)

with open("crates/apl_sema/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

