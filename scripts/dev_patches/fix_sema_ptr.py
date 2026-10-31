
with open("crates/apl_sema/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

compat_old = """            (Type::String, Type::Ptr) => true,
            (Type::Ptr, Type::I32) => true,
            (Type::I32, Type::Ptr) => true,"""

compat_new = """            (Type::String, Type::Ptr) => true,
            (Type::Ptr, Type::I32) => true,
            (Type::I32, Type::Ptr) => true,
            (Type::Slice(_), Type::Ptr) => true,
            (Type::Array(_, _), Type::Ptr) => true,
            (Type::Ref(_), Type::Ptr) => true,
            (Type::RefMut(_), Type::Ptr) => true,"""

content = content.replace(compat_old, compat_new)

with open("crates/apl_sema/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

