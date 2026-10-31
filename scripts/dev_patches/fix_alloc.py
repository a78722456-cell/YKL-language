
with open("crates/apl_sema/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

alloc_old = """        let alloc_array_func = Function {
            name: "alloc_array".to_string(),
            generics: vec!["T".to_string()],
            params: vec![Param {
                name: "size".to_string(),
                ty: Type::I32,
                span: Span::dummy(),
            }],
            return_type: Type::Slice(Box::new(Type::Named("T".to_string(), vec![]))),"""

alloc_new = """        let alloc_array_func = Function {
            name: "alloc_array".to_string(),
            generics: vec!["T".to_string()],
            params: vec![Param {
                name: "size".to_string(),
                ty: Type::I32,
                span: Span::dummy(),
            }, Param {
                name: "dummy".to_string(),
                ty: Type::Named("T".to_string(), vec![]),
                span: Span::dummy(),
            }],
            return_type: Type::Slice(Box::new(Type::Named("T".to_string(), vec![]))),"""

content = content.replace(alloc_old, alloc_new)

with open("crates/apl_sema/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

