
import re

with open("crates/apl_sema/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

# Replace substitute_type with monomorphize_type
new_func = """
    fn monomorphize_type(&mut self, ty: &Type, mapping: &HashMap<String, Type>) -> Type {
        match ty {
            Type::Named(name, args) => {
                if args.is_empty() {
                    if let Some(subst) = mapping.get(name) {
                        return subst.clone();
                    }
                }
                let new_args: Vec<Type> = args
                    .iter()
                    .map(|a| self.monomorphize_type(a, mapping))
                    .collect();
                
                if !new_args.is_empty() {
                    let mut type_name = name.clone();
                    for a in &new_args {
                        let arg_str = format!("{}", a).replace(" ", "").replace("<", "_").replace(">", "_").replace(",", "_");
                        type_name.push_str("_");
                        type_name.push_str(&arg_str);
                    }
                    
                    if !self.structs.contains_key(&type_name) && self.structs.contains_key(name) {
                        let orig = self.structs.get(name).unwrap().clone();
                        let mut new_struct = orig.clone();
                        new_struct.name = type_name.clone();
                        new_struct.generics = Vec::new();
                        
                        let mut local_map = HashMap::new();
                        for (i, g) in orig.generics.iter().enumerate() {
                            if let Some(a) = new_args.get(i) {
                                local_map.insert(g.clone(), a.clone());
                            }
                        }
                        for field in &mut new_struct.fields {
                            field.ty = self.monomorphize_type(&field.ty, &local_map);
                        }
                        self.structs.insert(type_name.clone(), new_struct);
                    } else if !self.enums.contains_key(&type_name) && self.enums.contains_key(name) {
                        let orig = self.enums.get(name).unwrap().clone();
                        let mut new_enum = orig.clone();
                        new_enum.name = type_name.clone();
                        new_enum.generics = Vec::new();
                        
                        let mut local_map = HashMap::new();
                        for (i, g) in orig.generics.iter().enumerate() {
                            if let Some(a) = new_args.get(i) {
                                local_map.insert(g.clone(), a.clone());
                            }
                        }
                        for variant in &mut new_enum.variants {
                            for ty in &mut variant.types {
                                *ty = self.monomorphize_type(ty, &local_map);
                            }
                        }
                        self.enums.insert(type_name.clone(), new_enum);
                    }
                    
                    return Type::Named(type_name, Vec::new());
                }
                Type::Named(name.clone(), new_args)
            }
            Type::Ref(inner) => Type::Ref(Box::new(self.monomorphize_type(inner, mapping))),
            Type::RefMut(inner) => Type::RefMut(Box::new(self.monomorphize_type(inner, mapping))),
            Type::Nullable(inner) => Type::Nullable(Box::new(self.monomorphize_type(inner, mapping))),
            Type::Array(inner, size) => Type::Array(Box::new(self.monomorphize_type(inner, mapping)), *size),
            Type::Slice(inner) => Type::Slice(Box::new(self.monomorphize_type(inner, mapping))),
            Type::Channel(inner) => Type::Channel(Box::new(self.monomorphize_type(inner, mapping))),
            _ => ty.clone(),
        }
    }
"""

# Find the start and end of substitute_type
start_idx = content.find("fn substitute_type(ty: &Type, mapping: &HashMap<String, Type>) -> Type {")
if start_idx != -1:
    end_idx = content.find("    fn analyze_expr_inner(&mut self, expr: &Expr) -> Type {", start_idx)
    content = content[:start_idx] + new_func + content[end_idx:]
    
    # Replace calls
    content = content.replace("Self::substitute_type", "self.monomorphize_type")

with open("crates/apl_sema/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)

print("Sema monomorphize done")

