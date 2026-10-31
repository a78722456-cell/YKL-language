
with open("crates/apl_sema/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

content = content.replace("""        if let Type::Named(name, args) = expected {
            if args.is_empty() && (name == "T" || name == "U" || name == "E") { // Hack for unresolved generics
                return true; 
            }
        }""", """        if let (Type::Named(n1, a1), Type::Named(n2, a2)) = (expected, actual) {
            if n1 == n2 && (a1.is_empty() || a2.is_empty()) { return true; }
        }
        if let Type::Named(name, args) = expected {
            if args.is_empty() && (name == "T" || name == "U" || name == "E") { // Hack for unresolved generics
                return true; 
            }
        }""")

with open("crates/apl_sema/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

