
with open("crates/apl_codegen/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

# I need to modify `offset_of_field` to strip `Ref` and `RefMut`
offset_func = """    fn offset_of_field(ty: &Type, field: &str, structs: &HashMap<String, StructDecl>, enums: &HashMap<String, EnumDecl>) -> usize {
        match ty {
            Type::Ref(inner) | Type::RefMut(inner) => Self::offset_of_field(inner.as_ref(), field, structs, enums),
            Type::Named(name, args) => {"""

content = content.replace("""    fn offset_of_field(ty: &Type, field: &str, structs: &HashMap<String, StructDecl>, enums: &HashMap<String, EnumDecl>) -> usize {
        match ty {
            Type::Named(name, args) => {""", offset_func)

with open("crates/apl_codegen/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

