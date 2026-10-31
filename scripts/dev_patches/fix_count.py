
with open("crates/apl_sema/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

open_c = content.count("{")
close_c = content.count("}")
print("open:", open_c, "close:", close_c)

