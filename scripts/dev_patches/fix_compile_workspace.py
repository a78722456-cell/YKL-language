
import re

with open("crates/apl_cli/src/main.rs", "r", encoding="utf-8") as f:
    content = f.read()

target1 = "fn compile_workspace(run_vm: bool) -> anyhow::Result<()> {"
new_target1 = "fn compile_workspace(run_vm: bool, input_file: Option<&str>) -> anyhow::Result<()> {"

target2 = """    let src_file = if Path::new("src/main.yk").exists() {
        "src/main.yk"
    } else if Path::new("test.yk").exists() {
        "test.yk"
    } else {
        // Create a dummy program in memory if no file exists
        "fn main() -> i32 { let a = 100; let b = 200; return a + b; }"
    };"""

new_target2 = """    let src_file = if let Some(path) = input_file {
        path.to_string()
    } else if Path::new("src/main.yk").exists() {
        "src/main.yk".to_string()
    } else if Path::new("test.yk").exists() {
        "test.yk".to_string()
    } else {
        "fn main() -> i32 { let a = 100; let b = 200; return a + b; }".to_string()
    };"""

target3 = """    let source = if src_file.ends_with(".yk") && Path::new(src_file).exists() {
        fs::read_to_string(src_file)?
    } else {
        src_file.to_string()
    };"""

new_target3 = """    let source = if src_file.ends_with(".yk") && Path::new(&src_file).exists() {
        fs::read_to_string(&src_file)?
    } else {
        src_file.clone()
    };"""

target4 = "compile_workspace(false)?;"
new_target4 = "compile_workspace(false, None)?;"

target5 = "compile_workspace(true)?;"
new_target5 = "compile_workspace(true, input.as_deref())?;"

content = content.replace(target1, new_target1).replace(target2, new_target2).replace(target3, new_target3).replace(target4, new_target4).replace(target5, new_target5)

with open("crates/apl_cli/src/main.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

