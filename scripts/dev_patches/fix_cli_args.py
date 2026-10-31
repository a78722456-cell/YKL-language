
import re

with open("crates/apl_cli/src/main.rs", "r", encoding="utf-8") as f:
    content = f.read()

target1 = """    Run {
        #[arg(long)]
        release: bool,
    },"""

new_target1 = """    Run {
        #[arg(long)]
        release: bool,
        input: Option<String>,
    },"""

target2 = """            Commands::Run { release } => {"""
new_target2 = """            Commands::Run { release, input } => {"""

target3 = """                let main_file = "test.yk";"""
new_target3 = """                let main_file = input.as_deref().unwrap_or("test.yk");"""

content = content.replace(target1, new_target1).replace(target2, new_target2).replace(target3, new_target3)

with open("crates/apl_cli/src/main.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

