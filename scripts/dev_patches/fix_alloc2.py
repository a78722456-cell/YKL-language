
with open("crates/apl_sema/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

alloc_old = """            }, Param {
                name: "dummy".to_string(),
                ty: Type::Named("T".to_string(), vec![]),
                span: Span::dummy(),
            }],
            return_type: Type::Slice(Box::new(Type::Named("T".to_string(), vec![]))),"""

alloc_new = """            }, Param {
                name: "dummy".to_string(),
                ty: Type::Ref(Box::new(Type::Named("T".to_string(), vec![]))),
                span: Span::dummy(),
            }],
            return_type: Type::Slice(Box::new(Type::Named("T".to_string(), vec![]))),"""

content = content.replace(alloc_old, alloc_new)

with open("crates/apl_sema/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

