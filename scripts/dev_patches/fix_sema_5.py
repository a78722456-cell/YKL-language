
import re

with open("crates/apl_sema/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

# Replace return Type::Named(type_name, Vec::new());
# With return Type::Named(name.clone(), new_args);
content = content.replace("return Type::Named(type_name, Vec::new());", "return Type::Named(name.clone(), new_args.clone());")

with open("crates/apl_sema/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

