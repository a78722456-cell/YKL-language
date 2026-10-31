
import re

with open("crates/apl_sema/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

missing_methods = """
    pub fn analyze_expr(&mut self, expr: &Expr) -> Type {
        let ty = self.analyze_expr_inner(expr);
        self.type_map.insert(expr.span, ty.clone());
        ty
    }

    fn infer_generics(expected: &Type, actual: &Type, mapping: &mut HashMap<String, Type>, generics: &[String]) {
        match (expected, actual) {
            (Type::Named(name, args1), Type::Named(_, args2)) => {
                if args1.is_empty() && generics.contains(name) {
                    mapping.insert(name.clone(), actual.clone());
                } else if args1.len() == args2.len() {
                    for (a1, a2) in args1.iter().zip(args2.iter()) {
                        Self::infer_generics(a1, a2, mapping, generics);
                    }
                }
            }
            (Type::Named(name, args), _) => {
                if args.is_empty() && generics.contains(name) {
                    mapping.insert(name.clone(), actual.clone());
                }
            }
            (Type::Ref(i1), Type::Ref(i2)) | (Type::RefMut(i1), Type::RefMut(i2)) | (Type::Nullable(i1), Type::Nullable(i2)) | (Type::Slice(i1), Type::Slice(i2)) | (Type::Channel(i1), Type::Channel(i2)) => {
                Self::infer_generics(i1, i2, mapping, generics);
            }
            (Type::Array(i1, s1), Type::Array(i2, s2)) if s1 == s2 => {
                Self::infer_generics(i1, i2, mapping, generics);
            }
            _ => {}
        }
    }

    fn is_compatible(&self, expected: &Type, actual: &Type) -> bool {
        if expected == actual {
            return true;
        }
        if let Type::Named(name, args) = expected {
            if args.is_empty() && (name == "T" || name == "U" || name == "E") { // Hack for unresolved generics
                return true; 
            }
        }
        if let (Type::Ref(i1), Type::RefMut(i2)) = (expected, actual) {
            return self.is_compatible(i1, i2);
        }
        false
    }
"""

start_idx = content.find("fn analyze_expr_inner")
if start_idx != -1:
    content = content[:start_idx] + missing_methods + content[start_idx:]

with open("crates/apl_sema/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

