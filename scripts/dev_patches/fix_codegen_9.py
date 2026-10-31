
with open("crates/apl_codegen/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

helpers = """
    fn mangle_name(name: &str, args: &[Type]) -> String {
        let mut n = name.to_string();
        for a in args {
            n.push_str("_");
            n.push_str(&format!("{}", a).replace(" ", "").replace("<", "_").replace(">", "_").replace(",", "_"));
        }
        n
    }
"""

content = content.replace("impl CCodeGenerator {", "impl CCodeGenerator {" + helpers)
content = content.replace("structs.get(name)", "structs.get(&Self::mangle_name(name, args))")
content = content.replace("enums.get(name)", "enums.get(&Self::mangle_name(name, args))")
content = content.replace("Type::Named(name, _)", "Type::Named(name, args)")
content = content.replace("format!(\"YKL_{}\", name.replace(\"::\", \"_\"))", "format!(\"YKL_{}\", Self::mangle_name(name, args).replace(\"::\", \"_\"))")

with open("crates/apl_codegen/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

