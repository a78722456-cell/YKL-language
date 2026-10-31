
with open("crates/apl_mir/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

bad = """    pub fn new_var_typed(&mut self, ty: Type) -> MirVar {
        let v = MirVar(self.var_counter);
        self.var_counter += 1;
        self.var_types.insert(v, ty);
        let v = MirVar(self.var_counter);
        self.var_counter += 1;
        v
    }"""

good = """    pub fn new_var_typed(&mut self, ty: Type) -> MirVar {
        let v = MirVar(self.var_counter);
        self.var_counter += 1;
        self.var_types.insert(v, ty);
        v
    }"""

content = content.replace(bad, good)

with open("crates/apl_mir/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

