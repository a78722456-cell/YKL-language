
with open("crates/apl_codegen/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

import re

fieldget_old = """                    MirInstruction::FieldGet(dest, base_val, field) => {
                        let base_ty = match base_val {
                            MirValue::Var(v) => func.var_types.get(v).unwrap(),
                            _ => &Type::I64,
                        };
                        let offset = Self::offset_of_field(base_ty, field, structs, enums);
                        let dest_ty = func.var_types.get(dest).unwrap_or(&Type::I64);
                        let size = Self::size_of_type(dest_ty, structs, enums);
                        if size == 1 {
                            code.push_str(&format!("    v_{} = {}.data[{}];\\n", dest.0, Self::val_to_c(base_val), offset));
                        } else {
                            code.push_str(&format!("    memcpy(&v_{}, &{}.data[{}], {} * sizeof(int64_t));\\n", dest.0, Self::val_to_c(base_val), offset, size));
                        }
                    }"""

fieldget_new = """                    MirInstruction::FieldGet(dest, base_val, field) => {
                        let base_ty = match base_val {
                            MirValue::Var(v) => func.var_types.get(v).unwrap(),
                            _ => &Type::I64,
                        };
                        let offset = Self::offset_of_field(base_ty, field, structs, enums);
                        let dest_ty = func.var_types.get(dest).unwrap_or(&Type::I64);
                        let size = Self::size_of_type(dest_ty, structs, enums);
                        
                        let (is_ref, inner_ty_str) = match base_ty {
                            Type::Ref(inner) | Type::RefMut(inner) => (true, Self::c_type_str(inner.as_ref(), structs, enums)),
                            _ => (false, "".to_string()),
                        };
                        
                        let base_access = if is_ref {
                            format!("(({}*){})->data", inner_ty_str, Self::val_to_c(base_val))
                        } else {
                            format!("{}.data", Self::val_to_c(base_val))
                        };
                        
                        if size == 1 {
                            code.push_str(&format!("    v_{} = {}[{}];\\n", dest.0, base_access, offset));
                        } else {
                            code.push_str(&format!("    memcpy(&v_{}, &{}[{}], {} * sizeof(int64_t));\\n", dest.0, base_access, offset, size));
                        }
                    }"""

content = content.replace(fieldget_old, fieldget_new)

fieldset_old = """                    MirInstruction::FieldSet(base_val, field, val) => {
                        let base_ty = match base_val {
                            MirValue::Var(v) => func.var_types.get(v).unwrap(),
                            _ => &Type::I64,
                        };
                        let offset = Self::offset_of_field(base_ty, field, structs, enums);
                        let val_ty = match val {
                            MirValue::Var(v) => func.var_types.get(v).unwrap(),
                            _ => &Type::I64,
                        };
                        let size = Self::size_of_type(val_ty, structs, enums);
                        if size == 1 {
                            code.push_str(&format!("    {}.data[{}] = {};\\n", Self::val_to_c(base_val), offset, Self::val_to_c(val)));
                        } else {
                            code.push_str(&format!("    memcpy(&{}.data[{}], &{}, {} * sizeof(int64_t));\\n", Self::val_to_c(base_val), offset, Self::val_to_c(val), size));
                        }
                    }"""

fieldset_new = """                    MirInstruction::FieldSet(base_val, field, val) => {
                        let base_ty = match base_val {
                            MirValue::Var(v) => func.var_types.get(v).unwrap(),
                            _ => &Type::I64,
                        };
                        let offset = Self::offset_of_field(base_ty, field, structs, enums);
                        let val_ty = match val {
                            MirValue::Var(v) => func.var_types.get(v).unwrap(),
                            _ => &Type::I64,
                        };
                        let size = Self::size_of_type(val_ty, structs, enums);
                        
                        let (is_ref, inner_ty_str) = match base_ty {
                            Type::Ref(inner) | Type::RefMut(inner) => (true, Self::c_type_str(inner.as_ref(), structs, enums)),
                            _ => (false, "".to_string()),
                        };
                        
                        let base_access = if is_ref {
                            format!("(({}*){})->data", inner_ty_str, Self::val_to_c(base_val))
                        } else {
                            format!("{}.data", Self::val_to_c(base_val))
                        };
                        
                        if size == 1 {
                            code.push_str(&format!("    {}[{}] = {};\\n", base_access, offset, Self::val_to_c(val)));
                        } else {
                            code.push_str(&format!("    memcpy(&{}[{}], &{}, {} * sizeof(int64_t));\\n", base_access, offset, Self::val_to_c(val), size));
                        }
                    }"""

content = content.replace(fieldset_old, fieldset_new)

with open("crates/apl_codegen/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

