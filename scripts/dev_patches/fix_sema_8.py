
with open("crates/apl_sema/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

content = content.replace("""if let (Type::I32, Type::Ptr) | (Type::I64, Type::Ptr) = (expected, actual) { return true; }""", """if let (Type::I32, Type::Ptr) | (Type::I64, Type::Ptr) = (expected, actual) { return true; }
        if let (Type::Ptr, Type::String) | (Type::String, Type::Ptr) = (expected, actual) { return true; }""")

with open("crates/apl_sema/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

