
import re

with open("crates/apl_sema/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

target = """                        } else {
                            if let Some((en_name, var_name)) = name.split_once("::") {"""
new_target = """                        } else {
                            if let Some((en_name, var_name)) = name.split_once("::") {"""

# Let's just insert } at the end of the `ExprKind::Call` block!
# Or we can just count the braces and insert one where it belongs!

