
with open("crates/apl_codegen/src/lib.rs", "r", encoding="utf-8") as f:
    lines = f.readlines()

for i, line in enumerate(lines):
    if "MirValue::ConstantString(s) => format!(" in line and "printf" in line:
        lines[i] = "                                    MirValue::ConstantString(s) => code.push_str(&format!(\"    printf(\\\"{}\\\\n\\\");\\n\", s)),\n"

with open("crates/apl_codegen/src/lib.rs", "w", encoding="utf-8") as f:
    f.writelines(lines)

