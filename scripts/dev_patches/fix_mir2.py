
with open("crates/apl_mir/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

content = content.replace("""                let dest = self.new_var();
                if func_name == "alloc_array" {""", """                let dest = self.new_var_typed(self.type_map.get(&expr.span).cloned().unwrap_or(Type::I64));
                if func_name == "alloc_array" {""")

content = content.replace("""                let res_var = self.new_var();
                self.push_inst(MirInstruction::Assign(res_var, then_res));""", """                let res_var = self.new_var_typed(self.type_map.get(&expr.span).cloned().unwrap_or(Type::I64));
                self.push_inst(MirInstruction::Assign(res_var, then_res));""")

with open("crates/apl_mir/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

