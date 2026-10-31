
import re

with open("crates/apl_sema/src/lib.rs", "r", encoding="utf-8") as f:
    content = f.read()

target = """                        self.errors.push(SemaError::UndefinedSymbol(name.clone(), expr.span));
                    }
                }
                Type::Void"""

new_target = """                        self.errors.push(SemaError::UndefinedSymbol(name.clone(), expr.span));
                        }
                    }
                }
                Type::Void"""

content = content.replace(target, new_target)

with open("crates/apl_sema/src/lib.rs", "w", encoding="utf-8") as f:
    f.write(content)
print("done")

