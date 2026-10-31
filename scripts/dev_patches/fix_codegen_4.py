
with open("crates/apl_codegen/src/lib.rs", "r", encoding="utf-8") as f:
    lines = f.readlines()

for i, line in enumerate(lines):
    if "MirValue::ConstantString(s) =>" in line:
        lines[i] = "                                    MirValue::ConstantString(s) => code.push_str(&format!(\"    printf(\\\"%s\\\\n\\\", \\\");\\n\", s)),\n"
    elif "Self::val_to_c(arg)" in line and "printf" in line:
        lines[i] = "                                    _ => code.push_str(&format!(\"    printf(\\\"%lld\\\\n\\\", (long long){});\\n\", Self::val_to_c(arg))),\n"

with open("crates/apl_codegen/src/lib.rs", "w", encoding="utf-8") as f:
    f.writelines(lines)

