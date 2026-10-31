
with open("crates/apl_codegen/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

content = content.replace("printf(\"{}\\\\n\");\\\\n", "printf(\\\"{}%s\\\\n\\\", \\\"\\\");\\\\n")
content = content.replace("printf(\"%lld\\\\n\", (long long){});\\\\n", "printf(\\\"%lld\\\\n\\\", (long long){});\\\\n")

with open("crates/apl_codegen/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)

