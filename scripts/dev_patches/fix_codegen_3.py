
import re
with open("crates/apl_codegen/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

content = re.sub(r"printf\(\\\"\{\}\\\\n\\\"\);\s*\\\\n\", s\)\)", r"printf(\"%%s\\\\n\", s))", content)
content = re.sub(r"printf\(\\\"\%lld\\\\n\\\", \(long long\)\{\}\);\s*\\\\n\", Self::val_to_c\(arg\)\)\)", r"printf(\"%%lld\\\\n\", (long long){}));", content)
content = content.replace("printf(\"{}\\\\n\");\\\\n", "printf(\\\"%s\\\\n\\\", s);\\n")
content = content.replace("printf(\"{}\\n\");\\n", "printf(\\\"%s\\\\n\\\", s);\\n")
content = content.replace("printf(\"%lld\\n\", (long long){});\\n", "printf(\\\"%lld\\\\n\\\", (long long){});\\n")
with open("crates/apl_codegen/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)

