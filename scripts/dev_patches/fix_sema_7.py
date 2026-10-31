
with open("crates/apl_sema/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

content = content.replace("""if args1.is_empty() && generics.contains(name) {
                    mapping.insert(name.clone(), actual.clone());""", """if args1.is_empty() && generics.contains(name) {
                    mapping.entry(name.clone()).or_insert(actual.clone());""")

content = content.replace("""if args.is_empty() && generics.contains(name) {
                    mapping.insert(name.clone(), actual.clone());""", """if args.is_empty() && generics.contains(name) {
                    mapping.entry(name.clone()).or_insert(actual.clone());""")

with open("crates/apl_sema/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

