
import re

with open("crates/apl_mir/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

content = content.replace("Type::Unknown", "Type::I64")
content = content.replace("v\n        let v = MirVar", "let v = MirVar")

with open("crates/apl_mir/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

