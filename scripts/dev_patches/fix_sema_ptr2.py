
with open("crates/apl_sema/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

compat_old = """            (Type::Slice(_), Type::Ptr) => true,
            (Type::Array(_, _), Type::Ptr) => true,"""

compat_new = """            (Type::Slice(_), Type::Ptr) => true,
            (Type::Array(_, _), Type::Ptr) => true,
            (Type::Ptr, Type::Slice(_)) => true,
            (Type::Ptr, Type::Array(_, _)) => true,"""

content = content.replace(compat_old, compat_new)

with open("crates/apl_sema/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

